import { initEmeraldBackground } from "@emerald-show/emerald-background";
import {
  hasNostrSigner,
  publishComment,
  publishCommentApproval,
  publishNostrPost,
  publishPostDeletion,
} from "./nostr.js";

const core = {
  initEmeraldBackground,
  hasNostrSigner,
  publishNostrPost,
  publishComment,
  publishCommentApproval,
  publishPostDeletion,
};

window.EmeraldBlogCore = core;
window.emeraldBlogCore = core;

initEmeraldBackground();

export {
  initEmeraldBackground,
  hasNostrSigner,
  publishNostrPost,
  publishComment,
  publishCommentApproval,
  publishPostDeletion,
};

