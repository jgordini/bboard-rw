import { test, expect, Page } from "@playwright/test";

const baseURL = "http://localhost:3000/";

const randomId = () => Math.random().toString(36).slice(2);

async function signup(page: Page) {
  const id = randomId();
  const name = `Test User ${id}`;
  const email = `test-${id}@example.com`;
  const password = `Password-${id}`;

  await page.goto(baseURL + "signup");
  await page.locator("#signup-name").fill(name);
  await page.locator("#signup-email").fill(email);
  await page.locator("#signup-password").fill(password);
  await page.getByRole("button", { name: "Sign up" }).click();

  return { name, email, password };
}

// ============================================================
// Board: sort tab switching
// ============================================================

test("sort tabs switch between Popular and Recent", async ({ page }) => {
  await page.goto(baseURL);

  const popularTab = page.locator("button.sort-tab", { hasText: "Popular" });
  const recentTab = page.locator("button.sort-tab", { hasText: "Recent" });

  await expect(popularTab).toBeVisible();
  await expect(recentTab).toBeVisible();

  // Popular is active by default
  await expect(popularTab).toHaveClass(/active/);
  await expect(recentTab).not.toHaveClass(/active/);

  // Click Recent
  await recentTab.click();
  await expect(recentTab).toHaveClass(/active/);
  await expect(popularTab).not.toHaveClass(/active/);

  // Switch back to Popular
  await popularTab.click();
  await expect(popularTab).toHaveClass(/active/);
  await expect(recentTab).not.toHaveClass(/active/);
});

// ============================================================
// Board: vote interaction (requires login)
// ============================================================

test("logged-in user can vote on an idea", async ({ page }) => {
  await signup(page);
  await page.goto(baseURL);

  // Submit an idea first
  await expect(page.getByRole("button", { name: "Post Idea" })).toBeVisible();
  await page.getByRole("button", { name: "Post Idea" }).click();

  const title = `Vote Test ${randomId()}`;
  await page.locator("#idea-title").fill(title);
  await page.locator("#idea-description").fill("Testing vote interaction.");
  await page.getByRole("button", { name: "Submit Idea" }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);

  // Find the idea's vote button
  const ideaItem = page.locator(".digg-item", { hasText: title }).first();
  await expect(ideaItem).toBeVisible();

  const voteBtn = ideaItem.locator(".digg-btn");
  await expect(voteBtn).toBeVisible();
  await voteBtn.click();

  // After voting the vote-box should have the voted class
  const voteBox = ideaItem.locator(".digg-vote-box");
  await expect(voteBox).toHaveClass(/voted/);
});

// ============================================================
// Board: search filters ideas
// ============================================================

test("search input filters displayed ideas", async ({ page }) => {
  await signup(page);
  await page.goto(baseURL);

  // Submit a distinctly named idea
  await page.getByRole("button", { name: "Post Idea" }).click();
  const uniqueTitle = `UniqueSearch ${randomId()}`;
  await page.locator("#idea-title").fill(uniqueTitle);
  await page.locator("#idea-description").fill("Searchable idea for testing.");
  await page.getByRole("button", { name: "Submit Idea" }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);

  await expect(page.locator(".digg-item", { hasText: uniqueTitle })).toBeVisible();

  // Search for something that won't match
  await page.locator("#idea-search").fill("zzzznonexistent");
  await expect(page.locator(".digg-item", { hasText: uniqueTitle })).toHaveCount(0);

  // Clear search to show all ideas again
  await page.locator("#idea-search").fill("");
  await expect(page.locator(".digg-item", { hasText: uniqueTitle })).toBeVisible();
});

// ============================================================
// Admin: requires authentication
// ============================================================

test("admin page requires login", async ({ page }) => {
  await page.goto(baseURL + "admin");

  // Non-logged-in users should see a login prompt or access denied
  const loginLink = page.getByRole("link", { name: "Go to Login" });
  const accessDenied = page.getByText("Access denied");
  const loginPrompt = page.getByText("Please log in");

  // Should see either a login prompt or access denied message
  await expect(loginPrompt.or(accessDenied).or(loginLink).first()).toBeVisible();
});

test("regular user is denied admin access", async ({ page }) => {
  await signup(page);
  await page.goto(baseURL + "admin");

  // Regular users should see access denied
  await expect(page.getByText("Access denied")).toBeVisible();
});

// ============================================================
// Responsive: mobile layout renders correctly
// ============================================================

test("mobile viewport renders sort tabs and sidebar", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto(baseURL);

  // Sort tabs should still be visible on mobile
  await expect(page.locator("button.sort-tab", { hasText: "Popular" })).toBeVisible();
  await expect(page.locator("button.sort-tab", { hasText: "Recent" })).toBeVisible();

  // Sidebar should be visible (moved above main content on mobile)
  await expect(page.locator(".sidebar")).toBeVisible();
});
