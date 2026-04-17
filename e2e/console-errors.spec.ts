import { test, expect, Page } from "@playwright/test";

/**
 * Collect all console errors and WASM panics while navigating the app.
 * Fails if any errors are found.
 */

function uniqueUser() {
  const id = Date.now().toString(36);
  return {
    username: `navtest_${id}`,
    email: `navtest_${id}@example.com`,
    password: "testpassword123",
  };
}

/** Attach a console listener that collects errors. */
function collectConsoleErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") {
      errors.push(`[console.error] ${msg.text()}`);
    }
  });
  page.on("pageerror", (err) => {
    errors.push(`[pageerror] ${err.message}`);
  });
  return errors;
}

test.describe("Console errors", () => {
  test("no errors on public pages", async ({ page }) => {
    const errors = collectConsoleErrors(page);

    const publicPages = ["/", "/login", "/register", "/search"];
    for (const path of publicPages) {
      await page.goto(path);
      await page.waitForLoadState("networkidle");
      // Small wait for any deferred WASM effects
      await page.waitForTimeout(500);
    }

    if (errors.length > 0) {
      console.log("Errors found on public pages:");
      errors.forEach((e) => console.log("  ", e));
    }
    expect(errors).toEqual([]);
  });

  test("no errors on authenticated pages", async ({ page }) => {
    const errors = collectConsoleErrors(page);
    const user = uniqueUser();

    // Register to get a session
    await page.goto("/register");
    await page.fill('input[name="username"]', user.username);
    await page.fill('input[name="display_name"]', "Nav Test");
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL("/collection");

    // Clear any registration errors before testing navigation
    errors.length = 0;

    const authPages = [
      "/collection",
      "/search",
      "/friends",
      "/settings",
      `/profile/${user.username}`,
    ];
    for (const path of authPages) {
      await page.goto(path);
      await page.waitForLoadState("networkidle");
      await page.waitForTimeout(500);
    }

    if (errors.length > 0) {
      console.log("Errors found on authenticated pages:");
      errors.forEach((e) => console.log("  ", e));
    }
    expect(errors).toEqual([]);
  });

  test("no errors during client-side navigation", async ({ page }) => {
    const errors = collectConsoleErrors(page);
    const user = uniqueUser();

    // Register
    await page.goto("/register");
    await page.fill('input[name="username"]', user.username);
    await page.fill('input[name="display_name"]', "Nav Test");
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL("/collection");
    errors.length = 0;

    // Click sidebar links (client-side navigation) — use .first() for desktop sidebar
    await page.locator('a[href="/friends"]').first().click();
    await expect(page).toHaveURL("/friends");
    await page.waitForTimeout(300);

    await page.locator('a[href="/collection"]').first().click();
    await expect(page).toHaveURL("/collection");
    await page.waitForTimeout(300);

    // Navigate via topbar user menu
    await page.locator("header button[title]").click();
    await page.locator('a[href="/settings"]').click();
    await expect(page).toHaveURL("/settings");
    await page.waitForTimeout(300);

    // Back to collection via sidebar
    await page.locator('a[href="/collection"]').first().click();
    await expect(page).toHaveURL("/collection");
    await page.waitForTimeout(300);

    // Navigate to profile via topbar
    await page.locator("header button[title]").click();
    const profileLink = page.locator(
      `a[href="/profile/${user.username}"]`
    );
    await profileLink.click();
    await expect(page).toHaveURL(`/profile/${user.username}`);
    await page.waitForTimeout(300);

    // Browser back
    await page.goBack();
    await page.waitForTimeout(300);

    if (errors.length > 0) {
      console.log("Errors found during client-side navigation:");
      errors.forEach((e) => console.log("  ", e));
    }
    expect(errors).toEqual([]);
  });
});
