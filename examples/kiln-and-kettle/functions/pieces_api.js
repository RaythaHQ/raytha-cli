/**
 * Public JSON feed of the shop: GET /pieces.json?form=mug&in_stock=true&maker=Margo%20Ashby
 */
function param(query, key) {
    var p = Array.from(query).find(function (q) { return q.Key === key; });
    return p ? p.Value[0] : null;
}

function get(query) {
    var filters = ["IsPublished eq 'true'"];
    var form = param(query, "form");
    if (form) filters.push("form eq '" + form.replace(/'/g, "") + "'");
    if (param(query, "in_stock") === "true") filters.push("in_stock eq 'true'");
    var result = API_V1.GetContentItems("pieces", "", "", filters.join(" and "), "title asc", 1, 100).Result;
    var site = CurrentOrganization.WebsiteUrl || "";
    var items = Array.from(result.Items).map(function (it) {
        var c = it.PublishedContent;
        return {
            title: it.PrimaryField,
            url: site + "/" + it.RoutePath,
            form: c.form,
            price_gbp: c.price,
            in_stock: c.in_stock,
            glaze: c.glaze,
            firing: c.firing,
            height_cm: c.height_cm,
        };
    });
    return new JsonResult({ shop: CurrentOrganization.OrganizationName, total: result.TotalCount, items: items });
}
