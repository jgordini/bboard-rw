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

test("signup shows logout in nav", async ({ page }) => {
  await signup(page);
  await expect(page.getByRole("button", { name: "Logout" })).toBeVisible();
});

test("profile updates after logout", async ({ page }) => {
  await signup(page);
  await page.goto(baseURL + "profile");
  await expect(page.getByText("Logged in as")).toBeVisible();

  await page.getByRole("button", { name: "Logout" }).click();
  await expect(page.getByText("You are not logged in.")).toBeVisible();
});

test("idea detail hides comment form after logout", async ({ page }) => {
  await signup(page);
  await page.goto(baseURL);

  await expect(page.getByRole("button", { name: "Post Idea" })).toBeVisible();
  await page.getByRole("button", { name: "Post Idea" }).click();

  const title = `Idea ${randomId()}`;
  await page.locator("#idea-title").fill(title);
  await page.locator("#idea-description").fill("This is a test idea.");
  await page.getByRole("button", { name: "Submit Idea" }).click();

  await expect(page.getByRole("dialog")).toHaveCount(0);

  const ideaLink = page.locator("a.spark-content", { hasText: title }).first();
  await expect(ideaLink).toBeVisible();
  await ideaLink.click();

  await expect(page.getByRole("heading", { name: "Add a Comment" })).toBeVisible();

  await page.getByRole("button", { name: "Logout" }).click();
  await expect(page.getByRole("heading", { name: "Add a Comment" })).toHaveCount(0);
});
