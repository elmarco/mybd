import { test, expect } from "@playwright/test";

function uniqueUser() {
  const id = Date.now().toString(36);
  return {
    username: `hydtest_${id}`,
    email: `hydtest_${id}@example.com`,
    password: "testpassword123",
  };
}

test("capture settings page SSR HTML", async ({ page }) => {
  const user = uniqueUser();

  // Register
  await page.goto("/register");
  await page.fill('input[name="username"]', user.username);
  await page.fill('input[name="display_name"]', "Hydration Test");
  await page.fill('input[name="email"]', user.email);
  await page.fill('input[name="password"]', user.password);
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL("/collection");

  // Disable JavaScript to get pure SSR HTML
  await page.context().route("**/*.js", (route) => route.abort());
  await page.context().route("**/*.wasm", (route) => route.abort());

  await page.goto("/settings");
  await page.waitForLoadState("domcontentloaded");

  // Get the main content HTML
  const mainHtml = await page.locator("main").innerHTML();
  console.log("=== SSR HTML (main) ===");
  console.log(mainHtml);
  console.log("=== END SSR HTML ===");

  // Check for comment nodes (markers) in the settings page div
  const commentNodes = await page.evaluate(() => {
    const main = document.querySelector("main");
    if (!main) return [];
    const walker = document.createTreeWalker(main, NodeFilter.SHOW_COMMENT);
    const comments: string[] = [];
    let node;
    while ((node = walker.nextNode())) {
      const parent = node.parentElement?.tagName || "?";
      comments.push(`[${parent}] <!-- ${(node as Comment).data} -->`);
    }
    return comments;
  });
  console.log("=== Comment markers ===");
  commentNodes.forEach((c) => console.log(" ", c));
  console.log(`Total: ${commentNodes.length} comment nodes`);
  console.log("=== END ===");

  expect(true).toBe(true); // Always pass — this is a debug test
});
