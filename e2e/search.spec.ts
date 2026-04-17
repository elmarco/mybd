import { test, expect, Page } from "@playwright/test";

function uniqueUser() {
  const id = Date.now().toString(36);
  return {
    username: `searchtest_${id}`,
    email: `searchtest_${id}@example.com`,
    password: "testpassword123",
  };
}

/** Register a new user and return the page on /collection. */
async function registerUser(page: Page) {
  const user = uniqueUser();
  await page.goto("/register");
  await page.fill('input[name="username"]', user.username);
  await page.fill('input[name="display_name"]', "Search Test");
  await page.fill('input[name="email"]', user.email);
  await page.fill('input[name="password"]', user.password);
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL("/collection");
  return user;
}

const searchInput = "#search-input";

test.describe("Search page", () => {
  test("navigating to /search?q= shows tabs with counts", async ({ page }) => {
    await page.goto("/search?q=asterix");
    // Wait for Suspense to resolve
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    const tabs = page.locator('[role="tab"]');
    await expect(tabs).toHaveCount(3);
    await expect(tabs.nth(0)).toContainText("Series");
    await expect(tabs.nth(1)).toContainText("Authors");
    await expect(tabs.nth(2)).toContainText("Users");
  });

  test("empty search shows nothing", async ({ page }) => {
    await page.goto("/search");
    await page.waitForLoadState("networkidle");
    // No tabs should be visible when query is empty
    await expect(page.locator('[role="tablist"]')).not.toBeVisible();
  });

  test("search bar reflects the URL query param", async ({ page }) => {
    await page.goto("/search?q=tintin");
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    const input = page.locator(searchInput);
    await expect(input).toHaveValue("tintin");
  });

  test("pressing Enter in search bar navigates to results", async ({
    page,
  }) => {
    await registerUser(page);

    const input = page.locator(searchInput);
    await input.fill("asterix");
    await input.press("Enter");

    await expect(page).toHaveURL(/\/search\?q=asterix/);
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });
    await expect(page.locator('[role="tab"]').first()).toBeVisible();
  });

  test("tab switching shows correct panels", async ({ page }) => {
    await page.goto("/search?q=asterix");
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    const seriesPanel = page.locator('[role="tabpanel"]').nth(0);
    const authorsPanel = page.locator('[role="tabpanel"]').nth(1);
    const usersPanel = page.locator('[role="tabpanel"]').nth(2);

    // Series tab active by default
    await expect(seriesPanel).toBeVisible();
    await expect(authorsPanel).not.toBeVisible();
    await expect(usersPanel).not.toBeVisible();

    // Click Authors tab
    await page.locator('[role="tab"]:has-text("Authors")').click();
    await expect(seriesPanel).not.toBeVisible();
    await expect(authorsPanel).toBeVisible();
    await expect(usersPanel).not.toBeVisible();

    // Click Users tab
    await page.locator('[role="tab"]:has-text("Users")').click();
    await expect(seriesPanel).not.toBeVisible();
    await expect(authorsPanel).not.toBeVisible();
    await expect(usersPanel).toBeVisible();

    // Click Series tab again
    await page.locator('[role="tab"]:has-text("Series")').click();
    await expect(seriesPanel).toBeVisible();
  });

  test("clear button clears the search input", async ({ page }) => {
    await registerUser(page);

    const input = page.locator(searchInput);
    await input.fill("test");
    await expect(input).toHaveValue("test");

    // Clear button appears when input has text
    const clearBtn = page.locator(
      'header button:has(span.material-symbols-outlined:text("close"))'
    );
    await expect(clearBtn).toBeVisible();
    await clearBtn.click();
    await expect(input).toHaveValue("");
  });
});

test.describe("Search history", () => {
  test("search history dropdown appears after searching", async ({ page }) => {
    await registerUser(page);

    const input = page.locator(searchInput);

    // Perform a search to add to history
    await input.fill("asterix");
    await input.press("Enter");
    await expect(page).toHaveURL(/\/search\?q=asterix/);
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    // Clear and focus the input — history dropdown should appear
    await input.fill("");
    await input.blur();
    await input.focus();

    const historyDropdown = page.locator(
      '[data-search-history] .absolute:has(span.material-symbols-outlined:text("history"))'
    );
    await expect(historyDropdown).toBeVisible({ timeout: 3000 });
    await expect(historyDropdown).toContainText("asterix");
  });

  test("clicking a history entry navigates to that search", async ({
    page,
  }) => {
    await registerUser(page);

    const input = page.locator(searchInput);

    // Search for something to build history
    await input.fill("asterix");
    await input.press("Enter");
    await expect(page).toHaveURL(/\/search\?q=asterix/);
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    // Navigate away
    await page.locator('a[href="/collection"]').first().click();
    await expect(page).toHaveURL("/collection");

    // Focus the search input and click the history entry
    await input.focus();
    const historyEntry = page.locator(
      '[data-search-history] .absolute button:has-text("asterix")'
    );
    await expect(historyEntry).toBeVisible({ timeout: 3000 });
    await historyEntry.click();

    await expect(page).toHaveURL(/\/search\?q=asterix/);
  });

  test("arrow keys navigate history dropdown and Enter selects", async ({
    page,
  }) => {
    await registerUser(page);

    const input = page.locator(searchInput);

    // Build history with two entries
    await input.fill("asterix");
    await input.press("Enter");
    await expect(page).toHaveURL(/\/search\?q=asterix/);
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    await input.fill("tintin");
    await input.press("Enter");
    await expect(page).toHaveURL(/\/search\?q=tintin/);
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    // Clear input (stays focused, dropdown opens via on:input)
    await input.fill("");

    const dropdown = page.locator('[data-search-history] .absolute');
    await expect(dropdown).toBeVisible({ timeout: 3000 });

    const buttons = dropdown.locator("button");
    // History is most-recent-first: tintin, asterix
    await expect(buttons).toHaveCount(2);

    // No item should be highlighted initially
    await expect(buttons.nth(0)).not.toHaveClass(/bg-gray-100/);
    await expect(buttons.nth(1)).not.toHaveClass(/bg-gray-100/);

    // ArrowDown highlights first item
    await input.press("ArrowDown");
    await expect(buttons.nth(0)).toHaveClass(/bg-gray-100/);
    await expect(buttons.nth(1)).not.toHaveClass(/bg-gray-100/);

    // ArrowDown again highlights second item
    await input.press("ArrowDown");
    await expect(buttons.nth(0)).not.toHaveClass(/bg-gray-100/);
    await expect(buttons.nth(1)).toHaveClass(/bg-gray-100/);

    // ArrowDown wraps back to first
    await input.press("ArrowDown");
    await expect(buttons.nth(0)).toHaveClass(/bg-gray-100/);

    // ArrowUp wraps to last
    await input.press("ArrowUp");
    await expect(buttons.nth(1)).toHaveClass(/bg-gray-100/);

    // Enter selects the highlighted entry (asterix)
    await input.press("Enter");
    await expect(page).toHaveURL(/\/search\?q=asterix/);
    // Dropdown should close
    await expect(dropdown).not.toBeVisible();
  });

  test("Escape closes dropdown and clears selection", async ({ page }) => {
    await registerUser(page);

    const input = page.locator(searchInput);

    // Build history
    await input.fill("asterix");
    await input.press("Enter");
    await expect(page).toHaveURL(/\/search\?q=asterix/);
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    // Clear input to open dropdown (stays focused)
    await input.fill("");

    const dropdown = page.locator('[data-search-history] .absolute');
    await expect(dropdown).toBeVisible({ timeout: 3000 });

    // Navigate down then Escape
    await input.press("ArrowDown");
    await expect(dropdown.locator("button").first()).toHaveClass(/bg-gray-100/);

    await input.press("Escape");
    await expect(dropdown).not.toBeVisible();
  });

  test("typing resets arrow key selection", async ({ page }) => {
    await registerUser(page);

    const input = page.locator(searchInput);

    // Build history
    await input.fill("asterix");
    await input.press("Enter");
    await expect(page).toHaveURL(/\/search\?q=asterix/);
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    // Clear input to open dropdown (stays focused)
    await input.fill("");

    const dropdown = page.locator('[data-search-history] .absolute');
    await expect(dropdown).toBeVisible({ timeout: 3000 });

    await input.press("ArrowDown");
    await expect(dropdown.locator("button").first()).toHaveClass(/bg-gray-100/);

    // Typing should clear the selection
    await input.pressSequentially("t");
    // If dropdown is still visible, no item should be highlighted
    const highlightedButtons = dropdown.locator("button.bg-gray-100");
    await expect(highlightedButtons).toHaveCount(0);
  });
});

test.describe("Search result panels", () => {
  test("series results show cards in a grid", async ({ page }) => {
    await page.goto("/search?q=asterix");
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    // The series panel should have a grid of cards
    const seriesPanel = page.locator('[role="tabpanel"]').first();
    const grid = seriesPanel.locator(
      ".grid.grid-cols-2"
    );
    // If there are series results, the grid should be visible
    const seriesCount = await page
      .locator('[role="tab"]')
      .first()
      .locator("span.rounded-full")
      .innerText();

    if (parseInt(seriesCount) > 0) {
      await expect(grid).toBeVisible();
    }
  });

  test("authors panel shows author cards with links", async ({ page }) => {
    await page.goto("/search?q=uderzo");
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    // Switch to authors tab
    await page.locator('[role="tab"]:has-text("Authors")').click();
    const authorsPanel = page.locator('[role="tabpanel"]').nth(1);
    await expect(authorsPanel).toBeVisible();

    const authorsCount = await page
      .locator('[role="tab"]:has-text("Authors")')
      .locator("span.rounded-full")
      .innerText();

    if (parseInt(authorsCount) > 0) {
      // Author entries should be links to /author/...
      const authorLink = authorsPanel.locator('a[href^="/author/"]').first();
      await expect(authorLink).toBeVisible();
    }
  });

  test("no console errors during search flow", async ({ page }) => {
    const errors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        errors.push(`[console.error] ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => {
      errors.push(`[pageerror] ${err.message}`);
    });

    await registerUser(page);

    const input = page.locator(searchInput);
    await input.fill("asterix");
    await input.press("Enter");
    await expect(page).toHaveURL(/\/search\?q=asterix/);
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });

    // Switch tabs
    await page.locator('[role="tab"]:has-text("Authors")').click();
    await page.waitForTimeout(300);
    await page.locator('[role="tab"]:has-text("Users")').click();
    await page.waitForTimeout(300);
    await page.locator('[role="tab"]:has-text("Series")').click();
    await page.waitForTimeout(300);

    // New search
    await input.fill("tintin");
    await input.press("Enter");
    await expect(page).toHaveURL(/\/search\?q=tintin/);
    await page.waitForSelector('[role="tablist"]', { timeout: 15000 });
    await page.waitForTimeout(500);

    if (errors.length > 0) {
      console.log("Errors found during search flow:");
      errors.forEach((e) => console.log("  ", e));
    }
    expect(errors).toEqual([]);
  });
});
