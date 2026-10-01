"""The people of Kiln & Kettle."""

MAKERS = {
    "margo": dict(
        name="Margo Ashby", role="master_potter", founder=True, joined="2014-03-01", years=31, glaze="#b5482a",
        crafts=["wheel", "wood_firing", "glaze_chemistry"],
        quote="A pot is finished when it stops asking you for more.",
        bio="<p>Margo trained for six years under a village potter in Shigaraki before coming home to Wales with a trunk of ash glaze recipes and a very bad back. In 2014 she rented the empty bobbin store at Ashby Mill and put a second-hand gas kiln in it.</p><p>She still throws every morning from six until the light changes, and she is the reason the studio smells of woodsmoke and Earl Grey. Her tall bottles and ember-red vases are in the Welsh Crafts Council collection.</p>",
        portrait=dict(bg="sand", skin="#c68863", hair="#2b1d17", style="bun", garment="#3b5d5a", apron="#b5703f", glasses=True),
    ),
    "tobias": dict(
        name="Tobias Okonkwo-Reed", role="glaze_chemist", founder=False, joined="2016-09-12", years=19, glaze="#27406b",
        crafts=["glaze_chemistry", "wheel", "slip_casting"],
        quote="Every glaze is a small argument between silica and fire. Fire usually wins.",
        bio="<p>Tobias was a materials scientist at a cement company until he discovered that melting rocks on purpose was much more fun. He runs the glaze lab: 640 test tiles and counting, each logged with its recipe and firing curve.</p><p>If you have ever wondered why a blue turns green in the same kiln load, he will tell you (at length) over a pot of oolong.</p>",
        portrait=dict(bg="sky", skin="#7a4e37", hair="#14100e", style="short", garment="#27406b", apron="#8a8f8a", glasses=False, beard="#1a1411"),
    ),
    "ines": dict(
        name="Ines Calloway", role="wheel_teacher", founder=False, joined="2017-01-23", years=14, glaze="#8fb7a6",
        crafts=["wheel", "handbuilding"],
        quote="Nobody centres clay on day one. Everybody centres clay by week four.",
        bio="<p>Ines teaches the Thursday beginners' course and has a rare gift for explaining what your hands are doing wrong without making you feel it. She has taught over nine hundred people to throw a mug, and keeps a shelf of the first wobbly attempts of every cohort.</p>",
        portrait=dict(bg="sage", skin="#e0b08c", hair="#7a3b22", style="long", garment="#c4572f", apron="#e8dcc4", glasses=False),
    ),
    "haruto": dict(
        name="Haruto Nakamura", role="tea_master", founder=False, joined="2018-05-05", years=26, glaze="#6f8f6a",
        crafts=["tea_ceremony", "kintsugi"],
        quote="Slow water makes sweet tea. Slow people make sweet pots.",
        bio="<p>Haruto grew up above his family's tea shop in Uji and moved to Wales for love, then stayed for the rain. He sources every leaf in the tea room himself and chooses which of the studio's cups each tea is served in, because a tea tastes different from a thin lip than a thick one.</p><p>On the first Sunday of the month he repairs broken favourites with gold lacquer. Bring a cracked cup, and a story.</p>",
        portrait=dict(bg="sage", skin="#d4a07a", hair="#d8d4cf", style="short", garment="#2b2926", apron="#6f8f6a", glasses=True),
    ),
    "priya": dict(
        name="Priya Venkataraman", role="studio_manager", founder=False, joined="2019-02-18", years=9, glaze="#e0a526",
        crafts=["handbuilding", "slip_casting"],
        quote="The best part of my job is the sound a kiln makes when it is opened and everything is fine.",
        bio="<p>Priya keeps the studio running: bookings, glaze orders, shipping, and the rota of who loads the kiln on Fridays. She came for a six-week hand-building course and never left. Her slip-cast lantern series sells out inside a day.</p>",
        portrait=dict(bg="ochre", skin="#a9714e", hair="#1a1210", style="curly", garment="#6a3a55", apron="#e0a526", glasses=False),
    ),
    "dewi": dict(
        name="Dewi Pritchard", role="apprentice", founder=False, joined="2023-09-04", years=3, glaze="#5a3320",
        crafts=["wood_firing", "wheel"],
        quote="I came to carry wood. I stayed because I couldn't stop looking at the flames.",
        bio="<p>Dewi is our apprentice and the keeper of the wood kiln, a hand-built anagama that fires twice a year for four days. He sleeps at the studio during firings and has strong feelings about oak versus ash.</p>",
        portrait=dict(bg="clay", skin="#e6bf9d", hair="#6b4a2b", style="short", garment="#4a6b4a", apron="#b5703f", glasses=False, beard="#6b4a2b"),
    ),
    "odessa": dict(
        name="Odessa Finch", role="master_potter", founder=False, joined="2020-06-15", years=22, glaze="#3f8f7e",
        crafts=["raku", "wood_firing", "handbuilding"],
        quote="Raku is the part of the job where I am allowed to shout.",
        bio="<p>Odessa runs the raku days, pulling glowing pots out of the kiln with tongs and dropping them into sawdust bins. Her crackled copper-green bowls are the most photographed objects in the studio. She trained in Santa Fe and in Bizen, and is equally comfortable with a bonfire or a spreadsheet.</p>",
        portrait=dict(bg="blush", skin="#8a5a40", hair="#d9d3c9", style="bob", garment="#2b2926", apron="#3f8f7e", glasses=False),
    ),
}
