import { test, expect, Page } from "@playwright/test";

// Known users created by `data_tool populate`
const MARC = {
  email: "marc@test.com",
  password: "password123",
  username: "marc",
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

// ---------------------------------------------------------------------------
// Author page
// ---------------------------------------------------------------------------
test.describe("Author page", () => {
  test("displays author name and birth year", async ({ page }) => {
    await page.goto("/author/christophe-bec");
    await expect(page.locator("h1")).toHaveText("Christophe Bec");
    await expect(page.locator("text=1969")).toBeVisible();
  });

  test("shows Series heading and series cards", async ({ page }) => {
    await page.goto("/author/christophe-bec");
    await expect(page.locator("h2", { hasText: "Series" })).toBeVisible();
    // Christophe Bec has series in the local DB (les-futurs-de-liu-cixin, carthago, etc.)
    // Series cards are <a data-nav-item> elements inside the grid
    const cards = page.locator("div.grid a[data-nav-item]");
    // wait for at least one series card to appear
    await expect(cards.first()).toBeVisible({ timeout: 10000 });
    expect(await cards.count()).toBeGreaterThanOrEqual(1);
  });

  test("shows 'Author not found' for unknown slug", async ({ page }) => {
    await page.goto("/author/nonexistent-author-slug");
    await expect(page.locator("text=Author not found")).toBeVisible();
  });

  test("edit link points to GitHub", async ({ page }) => {
    await page.goto("/author/christophe-bec");
    const editLink = page.locator('a[title="Edit data on GitHub"]');
    await expect(editLink).toBeVisible();
    await expect(editLink).toHaveAttribute(
      "href",
      /github\.com.*christophe-bec\.toml/
    );
  });
});

// ---------------------------------------------------------------------------
// Series detail page
// ---------------------------------------------------------------------------
test.describe("Series detail page", () => {
  test("displays series title, year, and album count", async ({ page }) => {
    await page.goto("/series/les-futurs-de-liu-cixin");
    await expect(page.locator("h1")).toHaveText("Les Futurs de Liu Cixin");
    await expect(page.locator("text=Year: 2022")).toBeVisible();
    await expect(page.locator("text=15 albums")).toBeVisible();
  });

  test("shows author tags linking to author pages", async ({ page }) => {
    await page.goto("/series/les-futurs-de-liu-cixin");
    const authorTag = page.locator('a[href="/author/christophe-bec"]');
    await expect(authorTag).toBeVisible({ timeout: 10000 });
    await expect(authorTag).toContainText("Christophe Bec");
  });

  test("shows album list for anonymous user", async ({ page }) => {
    await page.goto("/series/les-futurs-de-liu-cixin");
    // Albums section: should show individual album rows
    const albumRows = page.locator('[data-nav-item]');
    await expect(albumRows.first()).toBeVisible({ timeout: 10000 });
    // 15 albums in the series
    expect(await albumRows.count()).toBeGreaterThanOrEqual(15);
  });

  test("shows 'To be continued' for non-terminated series", async ({
    page,
  }) => {
    // les-futurs-de-liu-cixin has no is_terminated field → defaults to false
    await page.goto("/series/les-futurs-de-liu-cixin");
    await expect(
      page.locator("text=To be continued")
    ).toBeVisible({ timeout: 10000 });
  });

  test("shows select-all button when logged in", async ({ page }) => {
    await loginAs(page, MARC);
    await page.goto("/series/les-futurs-de-liu-cixin");
    await waitForHydration(page);

    // Marc owns all albums, so "Unselect all" should be visible
    await expect(
      page.locator("button", { hasText: /select all/i })
    ).toBeVisible({ timeout: 10000 });
  });

  test("shows series not found for unknown slug", async ({ page }) => {
    await page.goto("/series/nonexistent-series-slug");
    await expect(page.locator("text=Series not found")).toBeVisible();
  });

  test("shows description", async ({ page }) => {
    await page.goto("/series/les-futurs-de-liu-cixin");
    // The description starts with "Depuis la prévision"
    await expect(
      page.locator("text=Depuis la prévision")
    ).toBeVisible({ timeout: 10000 });
  });
});

// ---------------------------------------------------------------------------
// World map page
// ---------------------------------------------------------------------------
test.describe("World map page", () => {
  test("displays World Map heading", async ({ page }) => {
    await page.goto("/world");
    await expect(page.locator("h1", { hasText: "World Map" })).toBeVisible();
  });

  test("renders the map container", async ({ page }) => {
    await page.goto("/world");
    await waitForHydration(page);
    // The map div should exist and be visible
    await expect(page.locator("#world-map")).toBeVisible();
  });

  test("shows collector count", async ({ page }) => {
    await page.goto("/world");
    // Should show "N collector(s)" text
    await expect(
      page.locator("text=/\\d+ collectors?/")
    ).toBeVisible({ timeout: 10000 });
  });
});
