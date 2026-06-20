pub(super) const STYLE: &str = r#":root{--bg:#0b0b0d;--panel:#141416;--ink:#e8e3d6;--ink-mute:#9a958a;
--line:#2a2926;--accent:#e8472b}
*{box-sizing:border-box}
html{scroll-behavior:smooth}
body{margin:0;background:var(--bg);color:var(--ink);
font-family:"JetBrains Mono",ui-monospace,monospace;font-size:15px;
line-height:1.65;-webkit-font-smoothing:antialiased}
a{color:var(--accent);text-decoration:none}
a:hover{text-decoration:underline}
.wrap{max-width:820px;margin:0 auto;padding:32px 20px 80px}
header.site{display:flex;justify-content:space-between;align-items:center;
border-bottom:1px solid var(--line);padding-bottom:16px;margin-bottom:8px}
.brand{font-family:"Archivo Black",system-ui,sans-serif;letter-spacing:.04em}
.brand .bar{display:inline-block;width:22px;height:8px;background:var(--accent);
margin-right:8px;vertical-align:middle}
nav.crumb{font-size:12px;color:var(--ink-mute);letter-spacing:.12em;
text-transform:uppercase;margin:18px 0}
h1{font-family:"Fraunces",Georgia,serif;font-weight:500;font-size:2.1rem;
line-height:1.15;margin:.2em 0 .4em}
h2{font-family:"Fraunces",Georgia,serif;font-weight:500;font-size:1.4rem;
margin:2em 0 .5em;border-top:1px solid var(--line);padding-top:1.2em}
.lead{font-size:1.12rem;color:var(--ink);background:var(--panel);
border-left:3px solid var(--accent);padding:16px 18px;margin:1em 0}
.cta{display:inline-block;background:var(--accent);color:#fff;font-weight:700;
letter-spacing:.06em;text-transform:uppercase;padding:12px 22px;margin:18px 0;
border-radius:4px}
.cta:hover{text-decoration:none;filter:brightness(1.08)}
table{width:100%;border-collapse:collapse;margin:1em 0;font-size:14px}
th,td{text-align:left;padding:8px 10px;border-bottom:1px solid var(--line)}
th{color:var(--ink-mute);text-transform:uppercase;font-size:11px;
letter-spacing:.1em}
ul{padding-left:1.2em}
li{margin:.3em 0}
details{border:1px solid var(--line);border-radius:4px;padding:10px 14px;
margin:.5em 0;background:var(--panel)}
summary{cursor:pointer;font-weight:500}
.faq h2{border:0;padding:0;margin-top:1.6em}
footer.site{border-top:1px solid var(--line);margin-top:48px;padding-top:20px;
font-size:13px;color:var(--ink-mute)}
footer.site a{color:var(--ink-mute)}
.related{display:flex;flex-wrap:wrap;gap:10px;margin:10px 0}
.related a{border:1px solid var(--line);padding:6px 12px;border-radius:4px;
color:var(--ink)}
.src{font-size:12px;color:var(--ink-mute);margin-top:8px}"#;

pub(super) const FONTS: &str = "<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\" />\
<link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin />\
<link href=\"https://fonts.googleapis.com/css2?family=Archivo+Black&\
family=JetBrains+Mono:wght@300;400;500;700&\
family=Fraunces:ital,opsz,wght@0,9..144,400;0,9..144,500;1,9..144,500&\
display=swap\" rel=\"stylesheet\" />";
