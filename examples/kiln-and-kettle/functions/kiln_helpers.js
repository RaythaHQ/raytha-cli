/**
 * Liquid helpers for the Kiln & Kettle theme. Called from templates as
 *   {{ raytha_function("kiln_helpers", "money", amount=c.price_per_pot.Value) }}
 *   {{ raytha_function("kiln_helpers", "glaze_name", hex=c.glaze_colour.Value) }}
 */
function money(args) {
    var n = Number(args.amount || 0);
    return "\u00a3" + n.toFixed(2).replace(/\.00$/, "");
}

// The studio's named glazes. glaze_name() returns the one closest to a hex colour.
var GLAZES = [
    ["Rust", "#b5482a"], ["Indigo", "#27406b"], ["Celadon", "#8fb7a6"], ["Mustard", "#e0a526"], ["Oatmeal", "#e8dcc4"],
    ["Iron black", "#2b2926"], ["Ash green", "#6f8f6a"], ["Tenmoku", "#3a2a22"], ["Shino peach", "#e8a98a"], ["Slate blue", "#5e7a9a"],
    ["Chun blue", "#9eb6c9"], ["Honey ash", "#c9a15e"], ["Amber", "#c98a3a"], ["Forest", "#3f6b4a"], ["Copper green", "#3f8f7e"],
    ["Plum", "#6a3a55"], ["Midnight", "#1f2b4a"], ["Apple green", "#9db85a"], ["Heron grey", "#aeb4b4"], ["Smoked brown", "#6b3f2e"],
    ["Terracotta", "#c4572f"], ["Dune", "#d9b99b"], ["Peach blush", "#e8a98a"]
];

function rgb(hex) {
    var h = String(hex || "#000000").replace("#", "");
    return [parseInt(h.substr(0, 2), 16) || 0, parseInt(h.substr(2, 2), 16) || 0, parseInt(h.substr(4, 2), 16) || 0];
}

function glaze_name(args) {
    var want = rgb(args.hex), best = null, bestD = 1e12;
    for (var i = 0; i < GLAZES.length; i++) {
        var c = rgb(GLAZES[i][1]), d = 0;
        for (var k = 0; k < 3; k++) d += (want[k] - c[k]) * (want[k] - c[k]);
        if (d < bestD) { bestD = d; best = GLAZES[i][0]; }
    }
    return best;
}
