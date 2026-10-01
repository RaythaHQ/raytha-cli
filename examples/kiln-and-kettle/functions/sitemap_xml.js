/** /sitemap.xml built from published site pages and every content type with a public route. */
function esc(s) { return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;"); }

function get(query) {
    var site = (CurrentOrganization.WebsiteUrl || "").replace(/\/$/, "");
    var urls = [];
    Array.from(API_V1.GetSitePages("", "", 1, 200).Result.Items).forEach(function (p) {
        if (p.IsPublished) urls.push(site + "/" + p.RoutePath);
    });
    ["makers", "pieces", "workshops", "teas", "journal"].forEach(function (ct) {
        Array.from(API_V1.GetContentItems(ct, "", "", "IsPublished eq 'true'", "CreationTime desc", 1, 200).Result.Items)
            .forEach(function (it) { urls.push(site + "/" + it.RoutePath); });
    });
    var xml = '<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n' +
        urls.map(function (u) { return "  <url><loc>" + esc(u) + "</loc></url>"; }).join("\n") + "\n</urlset>";
    return new XmlResult(xml);
}
