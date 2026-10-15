import { test, expect } from "@playwright/test";

test("post response finishes before relay refresh and comments update afterward", async ({ page }) => {
  await page.route("**/emerald-core.js", (route) => route.fulfill({
    contentType: "application/javascript",
    body: "export {};",
  }));
  await page.goto("http://localhost:3000/", { waitUntil: "domcontentloaded" });
  const postPath = await page.locator('a[href^="/post/"]').first().getAttribute("href");
  expect(postPath).toBeTruthy();

  await page.route("**/api/seed_approved_comments*", async (route) => {
    await route.fulfill({
      json: [{
        id: "cached-comment-test",
        author: "Test author",
        display_name: "Test author",
        content: "A fresh comment loaded after the article.",
        created_at: "2026-10-15",
        created_at_secs: 1792022400,
      }],
    });
  });

  await page.goto(`http://localhost:3000${postPath}`, {
    waitUntil: "domcontentloaded",
    timeout: 5000,
  });
  await expect(page.locator("h1")).toBeVisible();
  await expect(page.locator('meta[property="og:title"]')).toHaveAttribute("content", /.+/);
  await expect(page.getByText("A fresh comment loaded after the article.")).toBeVisible();
});
