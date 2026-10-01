/** /llms.txt: a plain-text map of the site for language models. */
function section(title, type, line, order) {
    var res = API_V1.GetContentItems(type, "", "", "IsPublished eq 'true'", order || "CreationTime desc", 1, 100).Result;
    var out = ["", "## " + title];
    Array.from(res.Items).forEach(function (it) { out.push(line(it, it.PublishedContent)); });
    return out;
}

function get(query) {
    var site = CurrentOrganization.WebsiteUrl || "";
    var link = function (it) { return "[" + it.PrimaryField + "](" + site + "/" + it.RoutePath + ")"; };
    var lines = [
        "# " + CurrentOrganization.OrganizationName,
        "",
        "> A ceramics studio and tea room at Ashby Mill, Hay-on-Wye. We make and sell hand-thrown pottery, teach classes on the wheel, and serve tea in our own cups.",
        "",
        "## Pages",
    ];
    Array.from(API_V1.GetSitePages("", "", 1, 100).Result.Items).forEach(function (p) {
        if (p.IsPublished) lines.push("- [" + p.Title + "](" + site + "/" + p.RoutePath + ")");
    });
    lines = lines.concat(section("Shop", "pieces", function (it, c) {
        return "- " + link(it) + ": " + c.form + ", GBP " + c.price + (c.in_stock ? "" : " (sold)");
    }, "title asc"));
    lines = lines.concat(section("Classes", "workshops", function (it, c) {
        return "- " + link(it) + ": " + c.format + ", starts " + String(c.starts_on).substr(0, 10) + ", GBP " + c.price;
    }, "starts_on asc"));
    lines = lines.concat(section("Tea room", "teas", function (it, c) {
        return "- " + link(it) + ": " + c.tea_type + ", brew at " + c.steep_temp_c + "C";
    }, "title asc"));
    lines = lines.concat(section("Makers", "makers", function (it, c) {
        return "- " + link(it) + ": " + c.role;
    }, "name asc"));
    lines = lines.concat(section("Journal", "journal", function (it, c) {
        return "- " + link(it) + ": " + (c.excerpt || "");
    }));
    lines.push("", "## Machine readable", "- [Shop as JSON](" + site + "/pieces.json)", "- [Sitemap](" + site + "/sitemap.xml)");
    return new TextResult(lines.join("\n"));
}
