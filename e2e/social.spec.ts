import { test, expect, Page } from "@playwright/test";

// Known users created by `data_tool populate`
const MARC = {
  email: "marc@test.com",
  password: "password123",
  username: "marc",
};
const TOM = {
  email: "tom@test.com",
  password: "password123",
  username: "tom",
};

/** Wait for WASM hydration to complete (event handlers attached). */
async function waitForHydration(page: Page) {
  await page.waitForSelector("body[data-hydrated]", { timeout: 15000 });
}

/** Log in as a known populated user via the login page. */
async function loginAs(
  page: Page,
  user: { email: string; password: string }
) {
  await page.goto("/login");
  await page.fill('input[name="email"]', user.email);
  await page.fill('input[name="password"]', user.password);
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL("/collection");
}

/** Register a fresh unique user and land on /collection. */
async function registerUser(page: Page) {
  const id = Date.now().toString(36);
  const user = {
    username: `social_${id}`,
    email: `social_${id}@example.com`,
    password: "testpassword123",
    display_name: `Social ${id}`,
  };
  await page.goto("/register");
  await page.fill('input[name="username"]', user.username);
  await page.fill('input[name="display_name"]', user.display_name);
  await page.fill('input[name="email"]', user.email);
  await page.fill('input[name="password"]', user.password);
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL("/collection");
  return user;
}

// ─── Profile ────────────────────────────────────────────

test.describe("Profile", () => {
  test("view own profile shows name, stats, and collection", async ({
    page,
  }) => {
    await loginAs(page, MARC);
    await page.goto("/profile/marc");

    await expect(page.locator("h1")).toContainText("Marc");
    await expect(page.locator("text=@marc")).toBeVisible();

    // Stats should show album and lent counts
    await expect(page.locator("text=/\\d+ albums/")).toBeVisible();
    await expect(page.locator("text=/\\d+ lent/")).toBeVisible();

    // Collection section
    await expect(page.locator("h2:has-text('Collection')")).toBeVisible();
    await expect(page.locator(".grid")).toBeVisible();
  });

  test("view other public profile shows follow button", async ({ page }) => {
    await registerUser(page);
    await page.goto("/profile/marc");

    await expect(page.locator("h1")).toContainText("Marc");
    await expect(
      page.locator("button:has-text('Add friend')")
    ).toBeVisible();
  });

  test("private profile shows not found", async ({ page }) => {
    await page.goto("/profile/luc");

    await expect(page.locator("text=Profile not found")).toBeVisible();
    await expect(
      page.locator("text=This profile is private")
    ).toBeVisible();
  });

  test("follow and unfollow toggle on profile", async ({ page }) => {
    await registerUser(page);
    await page.goto("/profile/marc");

    // Follow
    const addBtn = page.locator("button:has-text('Add friend')");
    await expect(addBtn).toBeVisible();
    await addBtn.click();

    // Should change to "Remove friend"
    await expect(
      page.locator("button:has-text('Remove friend')")
    ).toBeVisible();

    // Unfollow
    await page.locator("button:has-text('Remove friend')").click();

    // Should change back to "Add friend"
    await expect(
      page.locator("button:has-text('Add friend')")
    ).toBeVisible();
  });
});

// ─── Friends Page ───────────────────────────────────────

test.describe("Friends page", () => {
  test("shows followed users", async ({ page }) => {
    await loginAs(page, MARC);
    await page.goto("/friends");

    await expect(page.locator("h1")).toContainText("Friends");
    // Marc follows Tom — use link selector to avoid strict-mode ambiguity
    await expect(page.locator('a[href="/profile/tom"]')).toBeVisible();
  });

  test("empty friends shows message", async ({ page }) => {
    await registerUser(page);
    await page.goto("/friends");

    await expect(page.locator("text=No friends yet")).toBeVisible();
  });

  test("add and remove friend from friends page", async ({ page }) => {
    await registerUser(page);

    // Follow marc from profile
    await page.goto("/profile/marc");
    await page.locator("button:has-text('Add friend')").click();
    await expect(
      page.locator("button:has-text('Remove friend')")
    ).toBeVisible();

    // Navigate to friends page
    await page.goto("/friends");
    await waitForHydration(page);
    await expect(page.locator('a[href="/profile/marc"]')).toBeVisible();

    // Remove friend
    await page
      .locator(
        'button:has(span.material-symbols-outlined:text("person_remove"))'
      )
      .click();

    // Friend card should disappear (page hides via local signal, not refetch)
    await expect(
      page.locator('a[href="/profile/marc"]')
    ).not.toBeVisible();
  });
});

// ─── Notifications ──────────────────────────────────────

test.describe("Notifications", () => {
  test("follow creates notification for target user", async ({ page }) => {
    // Register a new user and follow Marc
    await registerUser(page);
    await page.goto("/profile/marc");
    await page.locator("button:has-text('Add friend')").click();
    await expect(
      page.locator("button:has-text('Remove friend')")
    ).toBeVisible();

    // Clear session and login as Marc
    await page.context().clearCookies();
    await loginAs(page, MARC);
    // Reload to force SSR with authenticated session (unread_count resource
    // fires once during SSR and doesn't refetch after client-side login)
    await page.reload();

    // Wait for authenticated topbar to render
    await expect(page.locator("header button[title]")).toBeVisible();

    // Notification bell should show unread badge
    const badge = page.locator("[data-notif-dropdown] .bg-red-500");
    await expect(badge).toBeVisible({ timeout: 10000 });

    // Click the bell to open notification panel
    await page.locator("[data-notif-dropdown] button").first().click();
    const panel = page.locator("[data-notif-dropdown] .shadow-lg");
    await expect(panel).toBeVisible();
    await expect(panel.locator("text=Notifications")).toBeVisible();
  });

  test("opening notifications clears unread badge", async ({ page }) => {
    // Register user and follow marc (creates notification)
    await registerUser(page);
    await page.goto("/profile/marc");
    await page.locator("button:has-text('Add friend')").click();
    await expect(
      page.locator("button:has-text('Remove friend')")
    ).toBeVisible();

    // Login as Marc
    await page.context().clearCookies();
    await loginAs(page, MARC);
    // Reload to force SSR with authenticated session
    await page.reload();

    // Wait for authenticated topbar to render
    await expect(page.locator("header button[title]")).toBeVisible();

    const badge = page.locator("[data-notif-dropdown] .bg-red-500");
    await expect(badge).toBeVisible({ timeout: 10000 });

    // Open notifications (marks as read)
    await page.locator("[data-notif-dropdown] button").first().click();
    await expect(
      page.locator("[data-notif-dropdown] .shadow-lg")
    ).toBeVisible();

    // Reload to verify badge is gone
    await page.reload();
    await expect(page.locator("header button[title]")).toBeVisible();
    await expect(badge).not.toBeVisible();
  });
});

// ─── Collection ─────────────────────────────────────────

test.describe("Collection", () => {
  test("shows series from populated data", async ({ page }) => {
    await loginAs(page, MARC);
    // Already on /collection after login

    const grid = page.locator(".grid");
    await expect(grid).toBeVisible({ timeout: 15000 });

    // Marc owns albums from at least 2 series
    const items = grid.locator("> *");
    const count = await items.count();
    expect(count).toBeGreaterThanOrEqual(2);
  });
});

// ─── Album Detail ───────────────────────────────────────

test.describe("Album detail", () => {
  test("album from lent page shows lending info", async ({ page }) => {
    await loginAs(page, MARC);
    await page.goto("/lent");

    // Click on the first album link
    const albumLink = page.locator('a[href^="/album/"]').first();
    await expect(albumLink).toBeVisible({ timeout: 10000 });
    await albumLink.click();

    await expect(page).toHaveURL(/\/album\//);

    // Should show ownership status
    await expect(
      page.locator("button:has-text('In collection')")
    ).toBeVisible();

    // Should show lending info with borrower name
    await expect(page.locator("text=Lent to Tom")).toBeVisible();
  });

  test("keyboard left/right navigates between albums", async ({ page }) => {
    await loginAs(page, MARC);
    // Start on tome 2 (middle of series) so both arrows work
    await page.goto("/album/pour-que-respire-le-desert");
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });

    // Wait for navigation arrows to render (series_albums loaded)
    await expect(
      page.locator('a[aria-label="Next album"]')
    ).toBeVisible({ timeout: 10000 });

    // Press ArrowRight → should go to tome 3
    await page.keyboard.press("ArrowRight");
    await expect(page).toHaveURL(/les-trois-lois-du-monde/);
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });

    // Press ArrowRight again → tome 4
    await page.keyboard.press("ArrowRight");
    await expect(page).toHaveURL(/nourrir-l-humanite/);
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });

    // Press ArrowLeft → back to tome 3
    await page.keyboard.press("ArrowLeft");
    await expect(page).toHaveURL(/les-trois-lois-du-monde/);
  });

  test("keyboard nav does not jump to wrong album", async ({ page }) => {
    await loginAs(page, MARC);
    await page.goto("/album/la-perfection-du-cercle");
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });
    await expect(
      page.locator('a[aria-label="Next album"]')
    ).toBeVisible({ timeout: 10000 });

    // Rapidly press ArrowRight three times
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("ArrowRight");

    // Wait for the page to settle — URL should be exactly 3 ahead
    // from tome 5 (perfection-du-cercle) → 6 → 7 → 8 (brouillage-integral)
    // or it may stop at 6 if rapid presses are absorbed during loading.
    // The key assertion: URL must NOT jump backwards or to a wrong album.
    await page.waitForTimeout(2000);
    const url = page.url();
    // Should be on tome 6, 7, or 8 — never back on 5 or an unrelated album
    expect(url).toMatch(
      /proies-et-predateurs|l-attraction-de-la-foudre|brouillage-integral/
    );
  });

  test("click navigation arrows work correctly", async ({ page }) => {
    await loginAs(page, MARC);
    await page.goto("/album/pour-que-respire-le-desert");
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });

    // Click next arrow
    const nextArrow = page.locator('a[aria-label="Next album"]');
    await expect(nextArrow).toBeVisible({ timeout: 10000 });
    await nextArrow.click();
    await expect(page).toHaveURL(/les-trois-lois-du-monde/);
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });

    // Click prev arrow → back to tome 2
    const prevArrow = page.locator('a[aria-label="Previous album"]');
    await expect(prevArrow).toBeVisible({ timeout: 10000 });
    await prevArrow.click();
    await expect(page).toHaveURL(/pour-que-respire-le-desert/);
  });

  test("first album has no previous arrow, last has no next", async ({
    page,
  }) => {
    await loginAs(page, MARC);

    // First album in series
    await page.goto("/album/la-terre-vagabonde");
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });
    await expect(
      page.locator('a[aria-label="Next album"]')
    ).toBeVisible({ timeout: 10000 });
    await expect(
      page.locator('a[aria-label="Previous album"]')
    ).not.toBeVisible();

    // Last album in series (tome 15)
    await page.goto("/album/les-migrants-du-temps");
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });
    await expect(
      page.locator('a[aria-label="Previous album"]')
    ).toBeVisible({ timeout: 10000 });
    await expect(
      page.locator('a[aria-label="Next album"]')
    ).not.toBeVisible();
  });
});

// ─── Lent Albums ────────────────────────────────────────

test.describe("Lent albums", () => {
  test("lent page shows lent albums with borrower name", async ({
    page,
  }) => {
    await loginAs(page, MARC);
    await page.goto("/lent");

    await expect(page.locator("h1")).toContainText("Lent Albums");

    // Should show at least one lent album with Return button
    const returnBtn = page.locator("button:has-text('Return')").first();
    await expect(returnBtn).toBeVisible({ timeout: 10000 });

    // Should show "Lent to" and borrower link to Tom
    await expect(page.locator("text=Tom")).toBeVisible();
    await expect(page.locator('a[href="/profile/tom"]')).toBeVisible();
  });

  test("return album removes it from lent list", async ({ page }) => {
    await loginAs(page, MARC);
    await page.goto("/lent");

    const returnBtn = page.locator("button:has-text('Return')").first();
    await expect(returnBtn).toBeVisible({ timeout: 10000 });

    const countBefore = await page
      .locator("button:has-text('Return')")
      .count();

    // Return the first album
    await returnBtn.click();

    // The returned item is hidden; verify button count decreased
    await expect(
      page.locator("button:has-text('Return')")
    ).toHaveCount(countBefore - 1);
  });
});

// ─── Settings ───────────────────────────────────────────

test.describe("Settings", () => {
  test("update display name", async ({ page }) => {
    await registerUser(page);
    await page.goto("/settings");

    // Wait for the form to render
    const nameInput = page.locator('input[name="display_name"]');
    await expect(nameInput).toBeVisible({ timeout: 10000 });

    await nameInput.fill("Updated Name");
    // Ensure is_public checkbox is checked so the bool field is sent with form data
    await page.locator('input[name="is_public"]').check();
    await page.locator('button[type="submit"]:has-text("Save Changes")').click();

    await expect(
      page.locator("text=Profile updated successfully")
    ).toBeVisible({ timeout: 10000 });

    // Avatar button in header should reflect new name
    const avatarBtn = page.locator("header button[title]");
    await expect(avatarBtn).toHaveAttribute("title", "Updated Name");
  });

  test("update bio appears on profile", async ({ page }) => {
    const user = await registerUser(page);

    // Make profile public and set bio
    await page.goto("/settings");
    await expect(
      page.locator('input[name="display_name"]')
    ).toBeVisible({ timeout: 10000 });
    await page.locator('input[name="is_public"]').check();
    await page.locator('textarea[name="bio"]').fill("My test bio");
    await page.locator('button[type="submit"]:has-text("Save Changes")').click();
    await expect(
      page.locator("text=Profile updated successfully")
    ).toBeVisible({ timeout: 10000 });

    // Verify bio on profile page
    await page.goto(`/profile/${user.username}`);
    await expect(page.locator("text=My test bio")).toBeVisible();
  });

  test("making profile public makes it visible to others", async ({
    page,
  }) => {
    const user = await registerUser(page);

    // Make profile public
    await page.goto("/settings");
    await expect(
      page.locator('input[name="display_name"]')
    ).toBeVisible({ timeout: 10000 });
    await page.locator('input[name="is_public"]').check();
    await page.locator('button[type="submit"]:has-text("Save Changes")').click();
    await expect(
      page.locator("text=Profile updated successfully")
    ).toBeVisible({ timeout: 10000 });

    // Logout and view profile as anonymous
    await page.context().clearCookies();
    await page.goto(`/profile/${user.username}`);
    await expect(page.locator("h1")).toContainText(user.display_name);
  });
});

// ─── Sidebar Counts ────────────────────────────────────

test.describe("Sidebar counts", () => {
  test("collection count updates when toggling album ownership", async ({
    page,
  }) => {
    await loginAs(page, MARC);
    // Reload so SSR runs with authenticated session (count resources fire once during SSR)
    await page.reload();

    // Get initial sidebar collection count
    const countEl = page
      .locator('a[href="/collection"]')
      .first()
      .locator(".tabular-nums");
    await expect(countEl).not.toHaveText("0", { timeout: 10000 });
    const initial = Number(await countEl.textContent());
    expect(initial).toBeGreaterThan(0);

    // Navigate to an album and remove it from collection
    await page.goto("/album/la-terre-vagabonde");
    await waitForHydration(page);
    await expect(
      page.locator("button:has-text('In collection')")
    ).toBeVisible({ timeout: 10000 });
    await page.locator("button:has-text('In collection')").click();

    // Sidebar count should decrease without page reload
    await expect(countEl).toHaveText(String(initial - 1));

    // Add it back
    await expect(
      page.locator("button:has-text('Add to collection')")
    ).toBeVisible();
    await page.locator("button:has-text('Add to collection')").click();

    // Sidebar count should restore
    await expect(countEl).toHaveText(String(initial));
  });

  test("friends count updates when following/unfollowing", async ({
    page,
  }) => {
    await loginAs(page, MARC);
    // Reload so SSR runs with authenticated session
    await page.reload();

    // Get initial sidebar friends count
    const countEl = page
      .locator('a[href="/friends"]')
      .first()
      .locator(".tabular-nums");
    await expect(countEl).not.toHaveText("0", { timeout: 10000 });
    const initial = Number(await countEl.textContent());
    expect(initial).toBeGreaterThan(0);

    // Navigate to Tom's profile and unfollow
    await page.goto("/profile/tom");
    await waitForHydration(page);
    await expect(
      page.locator("button:has-text('Remove friend')")
    ).toBeVisible({ timeout: 10000 });
    await page.locator("button:has-text('Remove friend')").click();

    // Sidebar count should decrease without page reload
    await expect(countEl).toHaveText(String(initial - 1));

    // Follow back
    await expect(
      page.locator("button:has-text('Add friend')")
    ).toBeVisible();
    await page.locator("button:has-text('Add friend')").click();

    // Sidebar count should restore
    await expect(countEl).toHaveText(String(initial));
  });
});

// ─── User Search ────────────────────────────────────────

test.describe("User search", () => {
  test("finds public users in Users tab", async ({ page }) => {
    await loginAs(page, TOM);
    await page.goto("/search?q=marc");
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    // Switch to Users tab
    await page.locator('[role="tab"]:has-text("Users")').click();
    const usersPanel = page.locator('[role="tabpanel"]').nth(2);
    await expect(usersPanel).toBeVisible();

    // Marc should be in results with link to profile
    await expect(
      usersPanel.locator('a[href="/profile/marc"]')
    ).toBeVisible();
  });

  test("does not find private users", async ({ page }) => {
    await loginAs(page, MARC);
    await page.goto("/search?q=luc");
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    await page.locator('[role="tab"]:has-text("Users")').click();
    const usersPanel = page.locator('[role="tabpanel"]').nth(2);
    await expect(usersPanel).toBeVisible();

    // Luc should NOT appear (private user)
    await expect(
      usersPanel.locator("text=No users found")
    ).toBeVisible();
  });

  test("current user not shown in own search results", async ({
    page,
  }) => {
    await loginAs(page, MARC);
    await page.goto("/search?q=marc");
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    await page.locator('[role="tab"]:has-text("Users")').click();
    const usersPanel = page.locator('[role="tabpanel"]').nth(2);
    await expect(usersPanel).toBeVisible();

    // Marc should NOT find himself
    await expect(
      usersPanel.locator('a[href="/profile/marc"]')
    ).not.toBeVisible();
  });
});

// ─── Wishlist ──────────────────────────────────────────

test.describe("Wishlist", () => {
  test("can add and remove album from wishlist", async ({ page }) => {
    await loginAs(page, MARC);
    await page.reload();

    // Navigate to an album detail page
    await page.goto("/album/la-terre-vagabonde");
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });

    // Remove from collection if owned
    const inCollection = page.locator("button:has-text('In collection')");
    if (await inCollection.isVisible()) {
      await inCollection.click();
      await expect(
        page.locator("button:has-text('Add to collection')")
      ).toBeVisible();
    }

    // Add to wishlist
    const addWishlist = page.locator("button:has-text('Add to wishlist')");
    await expect(addWishlist).toBeVisible();
    await addWishlist.click();
    await expect(
      page.locator("button:has-text('On wishlist')")
    ).toBeVisible();

    // Remove from wishlist
    await page.locator("button:has-text('On wishlist')").click();
    await expect(
      page.locator("button:has-text('Add to wishlist')")
    ).toBeVisible();

    // Re-add to collection to restore state
    await page.locator("button:has-text('Add to collection')").click();
    await expect(
      page.locator("button:has-text('In collection')")
    ).toBeVisible();
  });

  test("owning a wishlisted album clears wishlist", async ({ page }) => {
    await loginAs(page, MARC);
    await page.reload();

    await page.goto("/album/la-terre-vagabonde");
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });

    // Ensure not owned
    const inCollection = page.locator("button:has-text('In collection')");
    if (await inCollection.isVisible()) {
      await inCollection.click();
      await expect(
        page.locator("button:has-text('Add to collection')")
      ).toBeVisible();
    }

    // Add to wishlist
    await page.locator("button:has-text('Add to wishlist')").click();
    await expect(
      page.locator("button:has-text('On wishlist')")
    ).toBeVisible();

    // Now add to collection — wishlist button should disappear
    await page.locator("button:has-text('Add to collection')").click();
    await expect(
      page.locator("button:has-text('In collection')")
    ).toBeVisible();
    // Wishlist button should not be visible when owned
    await expect(
      page.locator("button:has-text('On wishlist')")
    ).not.toBeVisible();
  });
});

// ─── For Sale ──────────────────────────────────────────

test.describe("For Sale", () => {
  test("can mark owned album for sale and remove listing", async ({
    page,
  }) => {
    await loginAs(page, MARC);
    await page.reload();

    // Navigate to an owned album
    await page.goto("/album/la-terre-vagabonde");
    await expect(page.locator("h1")).toBeVisible({ timeout: 10000 });

    // Ensure it's owned
    const addBtn = page.locator("button:has-text('Add to collection')");
    if (await addBtn.isVisible()) {
      await addBtn.click();
      await expect(
        page.locator("button:has-text('In collection')")
      ).toBeVisible();
    }

    // Click "Mark for sale"
    await page.locator("button:has-text('Mark for sale')").click();

    // Enter price and save
    await page.locator('input[type="number"]').fill("8.50");
    await page.locator("button:has-text('Save')").click();

    // Verify for-sale status shows
    await expect(page.locator("text=For sale at 8.50€")).toBeVisible();

    // Remove listing
    await page.locator("button:has-text('Remove')").click();
    await expect(
      page.locator("button:has-text('Mark for sale')")
    ).toBeVisible();
  });
});
