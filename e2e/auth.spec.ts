import { test, expect } from "@playwright/test";

// Generate a unique user for each test run to avoid collisions
function uniqueUser() {
  const id = Date.now().toString(36);
  return {
    username: `testuser_${id}`,
    email: `test_${id}@example.com`,
    password: "testpassword123",
  };
}

test.describe("Registration", () => {
  test("register with all fields and land on collection", async ({ page }) => {
    const user = uniqueUser();

    await page.goto("/register");
    await page.fill('input[name="username"]', user.username);
    await page.fill('input[name="display_name"]', "Test User");
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL("/collection");
    // Topbar should show the avatar link (authenticated), not "Sign In"
    await expect(page.locator("header button[title]")).toBeVisible();
    await expect(page.locator('text=Sign In')).not.toBeVisible();
  });

  test("register without display_name defaults to username", async ({
    page,
  }) => {
    const user = uniqueUser();

    await page.goto("/register");
    await page.fill('input[name="username"]', user.username);
    // Leave display_name empty
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL("/collection");
    // Avatar should show the first letter of the username
    const avatar = page.locator("header button[title]");
    await expect(avatar).toBeVisible();
    await expect(avatar).toHaveAttribute(
      "title",
      expect.stringContaining(user.username)
    );
  });

  test("register with duplicate email shows error", async ({ page }) => {
    const user = uniqueUser();

    // Register first time
    await page.goto("/register");
    await page.fill('input[name="username"]', user.username);
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL("/collection");

    // Clear cookies and try again with same email
    await page.context().clearCookies();
    await page.goto("/register");
    await page.fill('input[name="username"]', user.username + "2");
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');

    await expect(page.locator(".bg-red-50")).toContainText("already taken");
  });
});

test.describe("Login", () => {
  const user = uniqueUser();

  test.beforeAll(async ({ browser }) => {
    // Seed a user via registration
    const page = await browser.newPage();
    await page.goto("/register");
    await page.fill('input[name="username"]', user.username);
    await page.fill('input[name="display_name"]', "Login Test");
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL("/collection");
    await page.close();
  });

  test("login with valid credentials", async ({ page }) => {
    await page.goto("/login");
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL("/collection");
    await expect(page.locator("header button[title]")).toBeVisible();
    await expect(page.locator('text=Sign In')).not.toBeVisible();
  });

  test("login with wrong password shows error", async ({ page }) => {
    await page.goto("/login");
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', "wrongpassword");
    await page.click('button[type="submit"]');

    await expect(page.locator(".bg-red-50")).toContainText(
      "Invalid email or password"
    );
  });

  test("login with nonexistent email shows error", async ({ page }) => {
    await page.goto("/login");
    await page.fill('input[name="email"]', "nobody@example.com");
    await page.fill('input[name="password"]', "whatever123");
    await page.click('button[type="submit"]');

    await expect(page.locator(".bg-red-50")).toContainText(
      "Invalid email or password"
    );
  });
});

test.describe("Logout", () => {
  test("logout clears session", async ({ page }) => {
    const user = uniqueUser();

    // Register to get a session
    await page.goto("/register");
    await page.fill('input[name="username"]', user.username);
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL("/collection");
    await expect(page.locator("header button[title]")).toBeVisible();

    // Logout via the server endpoint
    await page.goto("/auth/logout");

    // Should be back on home page, unauthenticated
    await expect(page).toHaveURL("/");
    await expect(page.locator('header >> text=Sign In')).toBeVisible();
  });
});

test.describe("Delete Account", () => {
  test("delete button is disabled until confirmation phrase is typed", async ({
    page,
  }) => {
    const user = uniqueUser();

    // Register and go to settings via client-side navigation
    await page.goto("/register");
    await page.fill('input[name="username"]', user.username);
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL("/collection");
    await page.locator("header button[title]").click();
    await page.locator('a[href="/settings"]').click();
    await expect(page).toHaveURL("/settings");

    // Open the delete dialog
    await page.click('text=Delete Account');
    await expect(page.locator('text=Delete your account?')).toBeVisible();

    // Delete button should be disabled
    const deleteBtn = page.locator('button:has-text("Delete my account")');
    await expect(deleteBtn).toBeDisabled();

    // Type wrong phrase
    await page.fill('input[placeholder="Bachi-bouzouk"]', "wrong");
    await expect(deleteBtn).toBeDisabled();

    // Type correct phrase
    await page.fill('input[placeholder="Bachi-bouzouk"]', "Bachi-bouzouk");
    await expect(deleteBtn).toBeEnabled();
  });

  test("cancel closes dialog without deleting", async ({ page }) => {
    const user = uniqueUser();

    await page.goto("/register");
    await page.fill('input[name="username"]', user.username);
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL("/collection");
    await page.locator("header button[title]").click();
    await page.locator('a[href="/settings"]').click();
    await expect(page).toHaveURL("/settings");

    // Open and cancel
    await page.click('text=Delete Account');
    await expect(page.locator('text=Delete your account?')).toBeVisible();
    await page.click('button:has-text("Cancel")');
    await expect(page.locator('text=Delete your account?')).not.toBeVisible();

    // Still authenticated
    await expect(page.locator("header button[title]")).toBeVisible();
  });

  test("confirming deletion removes account and redirects home", async ({
    page,
  }) => {
    const user = uniqueUser();

    await page.goto("/register");
    await page.fill('input[name="username"]', user.username);
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL("/collection");
    await page.locator("header button[title]").click();
    await page.locator('a[href="/settings"]').click();
    await expect(page).toHaveURL("/settings");

    // Open dialog, type confirmation, submit
    await page.click('text=Delete Account');
    await page.fill('input[placeholder="Bachi-bouzouk"]', "Bachi-bouzouk");
    await page.click('button:has-text("Delete my account")');

    // Should redirect to home, unauthenticated
    await expect(page).toHaveURL("/");
    await expect(page.locator('header >> text=Sign In')).toBeVisible();

    // Trying to log in with deleted account should fail
    await page.goto("/login");
    await page.fill('input[name="email"]', user.email);
    await page.fill('input[name="password"]', user.password);
    await page.click('button[type="submit"]');
    await expect(page.locator(".bg-red-50")).toContainText(
      "Invalid email or password"
    );
  });
});
