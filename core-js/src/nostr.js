import { SimplePool } from "nostr-tools";
import { DEFAULT_RELAYS } from "./generated/nostr-relays.js";

const RELAY_TIMEOUT_MS = 8000;
const pool = new SimplePool({ enablePing: true });

export function hasNostrSigner() {
  return window.nostr != null;
}

export async function publishNostrPost(draft, options = {}) {
  const event = unsignedEvent(1, draft?.content ?? "", buildPostTags(draft));
  return signAndPublishEvent(event, options);
}

export async function publishComment(postId, content, options = {}) {
  const body = `${content ?? ""}`.trim();
  if (!body) {
    throw new Error("Write a comment before publishing.");
  }

  const tags = [
    ["E", postId, "", "root"],
    ["K", "1"],
    ["e", postId, "", "reply"],
    ["k", "1"],
  ];
  return signAndPublishEvent(unsignedEvent(1111, body, tags), options);
}

export async function publishCommentApproval(commentId, options = {}) {
  const tags = [
    ["e", commentId],
    ["L", "libnostrblog/moderation"],
    ["l", "approved", "libnostrblog/moderation"],
  ];
  return signAndPublishEvent(unsignedEvent(1985, "", tags), options);
}

export async function publishPostDeletion(postId, reason = "", expectedAuthor = "", options = {}) {
  if (!`${postId ?? ""}`.trim()) {
    throw new Error("Choose a post to delete.");
  }
  if (expectedAuthor && typeof expectedAuthor === "object") {
    options = expectedAuthor;
    expectedAuthor = "";
  }

  const tags = [
    ["e", postId],
    ["k", "1"],
  ];
  return signAndPublishEvent(unsignedEvent(5, reason.trim(), tags), {
    ...options,
    expectedPubkey: expectedAuthor,
  });
}

function unsignedEvent(kind, content, tags) {
  return {
    kind,
    created_at: Math.floor(Date.now() / 1000),
    content,
    tags,
  };
}

async function signAndPublishEvent(event, options = {}) {
  if (!hasNostrSigner()) {
    throw new Error("NIP-07 extension not found. Install or unlock a window.nostr signer.");
  }
  if (typeof window.nostr.signEvent !== "function") {
    throw new Error("window.nostr.signEvent is not callable.");
  }

  const signed = await window.nostr.signEvent(event);
  if (!signed.id) {
    throw new Error("NIP-07 signer returned an event without an id.");
  }
  assertSignedPubkey(signed, options.expectedPubkey);

  const relays = options.relays ?? DEFAULT_RELAYS;

  await publishToRelays(relays, signed);
  return signed.id;
}

function assertSignedPubkey(event, expectedPubkey) {
  const expected = `${expectedPubkey ?? ""}`.trim().toLowerCase();
  if (!expected) {
    return;
  }

  const actual = `${event.pubkey ?? ""}`.trim().toLowerCase();
  if (actual !== expected) {
    throw new Error(`NIP-07 signer pubkey ${actual} cannot delete posts by ${expected}.`);
  }
}

function buildPostTags(draft = {}) {
  const tags = [["title", `${draft.title ?? ""}`.trim()]];
  const excerpt = `${draft.excerpt ?? ""}`.trim();
  const featuredImage = `${draft.featured_image ?? draft.featuredImage ?? ""}`.trim();

  if (excerpt) {
    tags.push(["summary", excerpt]);
  }
  if (featuredImage) {
    tags.push(["image", featuredImage]);
  }

  `${draft.categories ?? ""}`
    .split(",")
    .map((category) => category.trim())
    .filter(Boolean)
    .forEach((category) => tags.push(["t", category]));

  return tags;
}

async function publishToRelays(relays, event) {
  const relayUrls = [...new Set(relays.map((relay) => `${relay}`.trim()).filter(Boolean))];

  try {
    await Promise.any(relayUrls.map((relay) => publishToRelay(relay, event)));
  } catch (error) {
    const errors = error.errors?.map((reason) => reason?.message ?? `${reason}`) ?? [
      error.message,
    ];
    throw new Error(`Relay publish failed: ${errors.join("; ")}`);
  }
}

async function publishToRelay(relayUrl, event) {
  try {
    const relay = await withRelayTimeout(
      pool.ensureRelay(relayUrl, { connectionTimeout: RELAY_TIMEOUT_MS }),
      relayUrl,
    );
    await withRelayTimeout(relay.publish(event), relayUrl);
  } catch (error) {
    throw new Error(`${relayUrl} publish failed: ${error?.message ?? error}`);
  } finally {
    pool.close([relayUrl]);
  }
}

function withRelayTimeout(promise, relayUrl) {
  let timeout;
  const timeoutPromise = new Promise((_, reject) => {
    timeout = setTimeout(() => {
      reject(new Error(`${relayUrl} timed out waiting for OK.`));
    }, RELAY_TIMEOUT_MS);
  });

  return Promise.race([promise, timeoutPromise]).finally(() => clearTimeout(timeout));
}
