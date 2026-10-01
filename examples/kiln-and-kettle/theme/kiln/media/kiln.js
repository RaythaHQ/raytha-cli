(function () {
  // reveal on scroll
  var io = "IntersectionObserver" in window ? new IntersectionObserver(function (es) {
    es.forEach(function (e) { if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); } });
  }, { threshold: 0.12 }) : null;
  document.querySelectorAll(".reveal,.curve").forEach(function (el) { io ? io.observe(el) : el.classList.add("in"); });

  // mobile nav
  var b = document.querySelector(".burger"), n = document.querySelector(".nav");
  if (b && n) b.addEventListener("click", function () { n.classList.toggle("open"); b.setAttribute("aria-expanded", n.classList.contains("open")); });

  // client-side filter chips: <div class="filters" data-filters="#grid" data-key="form"> with buttons data-f
  document.querySelectorAll("[data-filters]").forEach(function (bar) {
    var grid = document.querySelector(bar.dataset.filters), key = bar.dataset.key;
    bar.addEventListener("click", function (e) {
      var btn = e.target.closest("button"); if (!btn) return;
      bar.querySelectorAll("button").forEach(function (x) { x.classList.toggle("on", x === btn); });
      var f = btn.dataset.f;
      grid.querySelectorAll("[data-filter-item]").forEach(function (it) {
        var vals = (it.dataset[key] || "").split(" ");
        it.classList.toggle("gone", f !== "all" && vals.indexOf(f) < 0);
      });
    });
  });

  // counters
  document.querySelectorAll("[data-count]").forEach(function (el) {
    var to = parseFloat(el.dataset.count), done = false;
    var run = function () {
      if (done) return; done = true;
      var t0 = performance.now();
      (function tick(t) {
        var p = Math.min(1, (t - t0) / 1400), v = to * (1 - Math.pow(1 - p, 3));
        el.textContent = Math.round(v).toLocaleString();
        if (p < 1) requestAnimationFrame(tick);
      })(t0);
    };
    if (io) { new IntersectionObserver(function (es, o) { if (es[0].isIntersecting) { run(); o.disconnect(); } }).observe(el); } else run();
  });

  // steep timer
  document.querySelectorAll("[data-timer]").forEach(function (box) {
    var out = box.querySelector("output"), fg = box.querySelector(".fg"), btns = box.querySelectorAll(".inf button"), start = box.querySelector("[data-start]");
    var secs = 0, left = 0, iv = null, C = 408;
    function fmt(s) { return Math.floor(s / 60) + ":" + String(s % 60).padStart(2, "0"); }
    function set(s) { secs = left = s; out.textContent = fmt(s); fg.style.strokeDashoffset = 0; clearInterval(iv); iv = null; start.textContent = "Start steeping"; }
    btns.forEach(function (x) { x.addEventListener("click", function () { btns.forEach(function (y) { y.classList.toggle("on", y === x); }); set(+x.dataset.s); }); });
    if (btns[0]) btns[0].click();
    start.addEventListener("click", function () {
      if (iv) { set(secs); return; }
      start.textContent = "Reset";
      iv = setInterval(function () {
        left--; out.textContent = fmt(Math.max(left, 0)); fg.style.strokeDashoffset = C * (1 - left / secs);
        if (left <= 0) { clearInterval(iv); iv = null; out.textContent = "Ready"; start.textContent = "Steep again"; }
      }, 1000);
    });
  });

  // hours: highlight today
  var d = new Date().getDay();
  document.querySelectorAll("[data-dow]").forEach(function (r) { if (+r.dataset.dow === d) r.classList.add("today"); });
})();
