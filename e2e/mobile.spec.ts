import { test, expect, Page } from "@playwright/test";

// iPhone 13 viewport
const MOBILE_VIEWPORT = { width: 390, height: 844 };

const MARC = {
  email: "marc@test.com",
  password: "password123",
  username: "marc",
};

async function waitForHydration(page: Page) {
  await page.waitForSelector("body[data-hydrated]", { timeout: 15000 });
}

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

test.use({ viewport: MOBILE_VIEWPORT });

// ---------------------------------------------------------------------------
// Layout & navigation
// ---------------------------------------------------------------------------
test.describe("Mobile layout", () => {
  test("desktop sidebar is hidden, hamburger is visible", async ({ page }) => {
    await loginAs(page, MARC);
    await waitForHydration(page);

    // Desktop sidebar (md:flex) should not be visible at mobile width
    const sidebar = page.locator("aside.hidden");
    await expect(sidebar).toBeHidden();

    // Hamburger button should be visible in header
    const hamburger = page.locator('header button:has(span:text("menu"))');
    await expect(hamburger).toBeVisible();
  });

  test("hamburger opens and closes mobile drawer", async ({ page }) => {
    await loginAs(page, MARC);
    await waitForHydration(page);

    const hamburger = page.locator('header button:has(span:text("menu"))');
    await hamburger.click();

    // Mobile drawer aside should be visible
    const drawer = page.locator("div.fixed aside");
    await expect(drawer).toBeVisible();

    // Click backdrop to close
    const backdrop = page.locator("div.fixed > div.absolute.inset-0");
    await backdrop.click({ position: { x: 350, y: 400 } });
    await expect(drawer).toBeHidden();
  });

  test("mobile drawer navigates and auto-closes", async ({ page }) => {
    await loginAs(page, MARC);
    await waitForHydration(page);

    const hamburger = page.locator('header button:has(span:text("menu"))');
    await hamburger.click();

    const drawer = page.locator("div.fixed aside");
    await expect(drawer).toBeVisible();

    // Navigate to a different page (we're on /collection after login)
    const friendsLink = drawer.locator('a[href="/friends"]');
    await friendsLink.click();

    await expect(page).toHaveURL("/friends");
    // Drawer should auto-close after navigation
    await expect(drawer).toBeHidden();
  });

  test("guest sees sign-in instead of hamburger", async ({ page }) => {
    await page.goto("/");
    await waitForHydration(page);

    // Wait for Suspense in header to resolve (guest fallback shows Sign In)
    const signIn = page.locator("header >> text=Sign In");
    await expect(signIn).toBeVisible({ timeout: 10000 });

    // No hamburger for guests
    const hamburger = page.locator('header button:has(span:text("menu"))');
    await expect(hamburger).toHaveCount(0);
  });
});

// ---------------------------------------------------------------------------
// Dropdowns don't overflow viewport
// ---------------------------------------------------------------------------
test.describe("Mobile dropdowns", () => {
  test("user menu fits within viewport", async ({ page }) => {
    await loginAs(page, MARC);
    await waitForHydration(page);

    // Open user menu
    const avatarButton = page.locator("[data-user-menu] button");
    await avatarButton.click();

    const menu = page.locator("[data-user-menu] > div.absolute");
    await expect(menu).toBeVisible();

    const box = await menu.boundingBox();
    expect(box).toBeTruthy();
    expect(box!.x).toBeGreaterThanOrEqual(0);
    expect(box!.x + box!.width).toBeLessThanOrEqual(MOBILE_VIEWPORT.width);
  });

  test("notification dropdown fits within viewport", async ({ page }) => {
    await loginAs(page, MARC);
    await waitForHydration(page);

    // Open notification dropdown
    const bellButton = page.locator("[data-notif-dropdown] button");
    await bellButton.click();

    const dropdown = page.locator("[data-notif-dropdown] > div.absolute");
    await expect(dropdown).toBeVisible();

    const box = await dropdown.boundingBox();
    expect(box).toBeTruthy();
    expect(box!.x).toBeGreaterThanOrEqual(0);
    expect(box!.x + box!.width).toBeLessThanOrEqual(MOBILE_VIEWPORT.width);
  });
});

// ---------------------------------------------------------------------------
// Touch targets
// ---------------------------------------------------------------------------
test.describe("Mobile touch targets", () => {
  test("interactive header buttons meet minimum size", async ({ page }) => {
    await loginAs(page, MARC);
    await waitForHydration(page);

    // Notification bell
    const bell = page.locator("[data-notif-dropdown] button");
    const bellBox = await bell.boundingBox();
    expect(bellBox).toBeTruthy();
    expect(bellBox!.width).toBeGreaterThanOrEqual(36);
    expect(bellBox!.height).toBeGreaterThanOrEqual(36);

    // Hamburger
    const hamburger = page.locator('header button:has(span:text("menu"))');
    const hamBox = await hamburger.boundingBox();
    expect(hamBox).toBeTruthy();
    expect(hamBox!.width).toBeGreaterThanOrEqual(36);
    expect(hamBox!.height).toBeGreaterThanOrEqual(36);
  });
});

// ---------------------------------------------------------------------------
// Search on mobile
// ---------------------------------------------------------------------------
test.describe("Mobile search", () => {
  test("search bar is usable and results load", async ({ page }) => {
    await page.goto("/");
    await waitForHydration(page);

    const input = page.locator("#search-input");
    await expect(input).toBeVisible();

    await input.fill("asterix");
    await input.press("Enter");

    await expect(page).toHaveURL(/\/search\?q=asterix/);
    // Tab bar should be visible
    await expect(page.locator('[role="tablist"]')).toBeVisible();
  });

  test("search results grid adapts to mobile width", async ({ page }) => {
    await page.goto("/search?q=asterix");
    await waitForHydration(page);

    // Wait for results to load
    const grid = page.locator('[role="tabpanel"]:not(.hidden) div.grid');
    await expect(grid.first()).toBeVisible({ timeout: 10000 });

    // On mobile (390px), grid should fill available width (not overflow)
    const box = await grid.first().boundingBox();
    expect(box).toBeTruthy();
    expect(box!.width).toBeLessThanOrEqual(MOBILE_VIEWPORT.width);
  });
});

// ---------------------------------------------------------------------------
// Pages render without horizontal overflow
// ---------------------------------------------------------------------------
test.describe("Mobile no horizontal overflow", () => {
  const pages = [
    { name: "home", path: "/" },
    { name: "search", path: "/search?q=asterix" },
    { name: "series", path: "/series/les-futurs-de-liu-cixin" },
    { name: "author", path: "/author/christophe-bec" },
  ];

  for (const { name, path } of pages) {
    test(`${name} page has no horizontal scroll`, async ({ page }) => {
      await page.goto(path);
      await waitForHydration(page);

      const hasHorizontalScroll = await page.evaluate(
        () => document.documentElement.scrollWidth > document.documentElement.clientWidth
      );
      expect(hasHorizontalScroll).toBe(false);
    });
  }

  test("collection page has no horizontal scroll", async ({ page }) => {
    await loginAs(page, MARC);
    await waitForHydration(page);

    const hasHorizontalScroll = await page.evaluate(
      () => document.documentElement.scrollWidth > document.documentElement.clientWidth
    );
    expect(hasHorizontalScroll).toBe(false);
  });
});
