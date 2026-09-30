/* Aurora Observatory theme behaviour. Vanilla JS, progressive enhancement only. */
(function () {
  var reduce = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  /* nav: solid background after scroll, mobile menu */
  var nav = document.querySelector(".nav");
  function onScroll() { if (nav) nav.classList.toggle("scrolled", window.scrollY > 24); }
  onScroll();
  window.addEventListener("scroll", onScroll, { passive: true });
  var tg = document.querySelector(".nav-toggle"), links = document.querySelector(".nav-links");
  if (tg && links) tg.addEventListener("click", function () { var o = links.classList.toggle("open"); tg.setAttribute("aria-expanded", o); });

  /* starfield */
  var cv = document.getElementById("stars");
  if (cv && cv.getContext && !reduce) {
    var ctx = cv.getContext("2d"), stars = [], W, H, dpr = Math.min(window.devicePixelRatio || 1, 2);
    function size() {
      W = cv.width = innerWidth * dpr; H = cv.height = innerHeight * dpr;
      stars = []; var n = Math.floor((innerWidth * innerHeight) / 5200);
      for (var i = 0; i < n; i++) stars.push({ x: Math.random() * W, y: Math.random() * H, r: (Math.random() * 1.2 + 0.25) * dpr, p: Math.random() * 6.28, s: Math.random() * 0.02 + 0.004 });
    }
    function draw(t) {
      ctx.clearRect(0, 0, W, H);
      for (var i = 0; i < stars.length; i++) {
        var s = stars[i], a = 0.35 + 0.65 * Math.abs(Math.sin(s.p + t * s.s * 0.06));
        ctx.globalAlpha = a; ctx.fillStyle = "#fff"; ctx.beginPath(); ctx.arc(s.x, s.y, s.r, 0, 6.283); ctx.fill();
      }
      requestAnimationFrame(draw);
    }
    size(); addEventListener("resize", size); requestAnimationFrame(draw);
  }

  /* reveal on scroll + counters + dial */
  var io = "IntersectionObserver" in window ? new IntersectionObserver(function (es) {
    es.forEach(function (e) {
      if (!e.isIntersecting) return;
      e.target.classList.add("in"); io.unobserve(e.target);
      if (e.target.matches("[data-count]")) countUp(e.target);
    });
  }, { threshold: 0.15 }) : null;
  function countUp(el) {
    var to = parseFloat(el.getAttribute("data-count")), suf = el.getAttribute("data-suffix") || "", dec = (String(to).split(".")[1] || "").length, t0 = null;
    if (reduce) { el.textContent = to.toLocaleString() + suf; return; }
    function step(t) { if (!t0) t0 = t; var p = Math.min((t - t0) / 1600, 1), e = 1 - Math.pow(1 - p, 4); el.textContent = (to * e).toLocaleString(undefined, { minimumFractionDigits: dec, maximumFractionDigits: dec }) + suf; if (p < 1) requestAnimationFrame(step); }
    requestAnimationFrame(step);
  }
  document.querySelectorAll(".reveal, .dial, [data-count]").forEach(function (el, i) {
    if (!io) { el.classList.add("in"); if (el.matches("[data-count]")) countUp(el); return; }
    io.observe(el);
  });

  /* card spotlight follows the pointer */
  document.addEventListener("pointermove", function (e) {
    var c = e.target.closest && e.target.closest(".card");
    if (!c) return;
    var r = c.getBoundingClientRect();
    c.style.setProperty("--mx", e.clientX - r.left + "px"); c.style.setProperty("--my", e.clientY - r.top + "px");
  }, { passive: true });

  /* horizontal rails */
  document.querySelectorAll("[data-rail]").forEach(function (box) {
    var rail = box.querySelector(".rail");
    box.querySelectorAll("[data-dir]").forEach(function (b) {
      b.addEventListener("click", function () { rail.scrollBy({ left: Number(b.getAttribute("data-dir")) * 470, behavior: "smooth" }); });
    });
  });

  /* client-side filter chips */
  document.querySelectorAll("[data-filters]").forEach(function (bar) {
    var scope = document.querySelector(bar.getAttribute("data-filters")) || document;
    bar.addEventListener("click", function (e) {
      var b = e.target.closest("button"); if (!b) return;
      bar.querySelectorAll("button").forEach(function (x) { x.classList.toggle("on", x === b); });
      var f = b.getAttribute("data-f"), key = bar.getAttribute("data-key") || "cat";
      scope.querySelectorAll("[data-filter-item]").forEach(function (it) {
        var vals = (it.getAttribute("data-" + key) || "").split(" ");
        it.classList.toggle("hide", f !== "all" && vals.indexOf(f) < 0);
      });
    });
  });
})();
