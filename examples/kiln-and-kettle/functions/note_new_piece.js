/** content_item_created: when a piece is listed, record it in the background-task log. */
function run(payload) {
    if (payload.ContentType.DeveloperName !== "pieces") return;
    // Nothing to send in a demo: the call proves the event fires and sees the new item's fields.
    var c = payload.PublishedContent || payload.DraftContent || {};
    if (!c.price) throw new Error("New piece '" + payload.PrimaryField + "' has no price");
}
