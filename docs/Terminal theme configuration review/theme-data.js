/* aibox theme audit — shared data + derivation engine.
   Consumed by the HTML review artifact and by the corrections generator.
   Slot order: bg fg accent green red yellow orange cyan muted */

export const SLOTS = ["bg", "fg", "accent", "green", "red", "yellow", "orange", "cyan", "muted"];

/* CURRENT — transcribed verbatim from THEME-DESIGN-AGENT-BRIEFING.md (v0.x snapshot, 2026-08-20).
   [id, family, displayName, mode, variantKey, bg, fg, accent, green, red, yellow, orange, cyan, muted] */
const C = [
["Andromeeda","andromeeda","Andromeeda","dark","solo","#23262E","#D5CED9","#00E8C6","#89E044","#EE5D43","#FFCC00","#F39C12","#00E8C6","#6B6B6B"],
["AuroraX","aurora-x","AuroraX","dark","solo","#07090F","#D4D4D4","#569CD6","#B5CEA8","#F44747","#CE9178","#CE9178","#4EC9B0","#5C6370"],
["AyuDark","ayu","AyuDark","dark","default","#0A0E14","#B3B1AD","#39BAE6","#AAD94C","#F07178","#FFB454","#FF8F40","#95E6CB","#626A73"],
["AyuMirage","ayu","AyuMirage","dark","mirage","#1F2430","#CCCAC2","#5CCFE6","#BAE67E","#F28779","#FFD173","#FFAD66","#95E6CB","#707A8C"],
["AyuLight","ayu","AyuLight","light","default","#FAFAFA","#5C6773","#55B4D4","#86B300","#E7676A","#FA8D3E","#F07171","#4CBF99","#ABB0B6"],
["CatppuccinMocha","catppuccin","CatppuccinMocha","dark","default","#1E1E2E","#CDD6F4","#89B4FA","#A6E3A1","#F38BA8","#F9E2AF","#FAB387","#94E2D5","#6C7086"],
["CatppuccinMacchiato","catppuccin","CatppuccinMacchiato","dark","macchiato","#24273A","#CAD3F5","#8AADF4","#A6DA95","#ED8796","#EED49F","#F5A97F","#8BD5CA","#6E738D"],
["CatppuccinFrappe","catppuccin","CatppuccinFrappe","dark","frappe","#303446","#C6D0F5","#8CAAEE","#A6D189","#E78284","#E5C890","#EF9F76","#81C8BE","#737994"],
["CatppuccinLatte","catppuccin","CatppuccinLatte","light","default","#EFF1F5","#4C4F69","#1E66F5","#40A02B","#D20F39","#DF8E1D","#FE640B","#179299","#9CA0B0"],
["Dracula","dracula","Dracula","dark","default","#282A36","#F8F8F2","#BD93F9","#50FA7B","#FF5555","#F1FA8C","#FFB86C","#8BE9FD","#6272A4"],
["DraculaSoft","dracula","DraculaSoft","dark","soft","#22212C","#F8F8F2","#C8A8F9","#62E884","#E76D6D","#E9E987","#FFCA80","#A1F0FE","#7970A9"],
["EverforestDark","everforest","EverforestDark","dark","default","#2D353B","#D3C6AA","#7FBBB3","#A7C080","#E67E80","#DBBC7F","#D699B6","#83C092","#7A8478"],
["EverforestLight","everforest","EverforestLight","light","default","#FDF6E3","#5C6A72","#3A94C5","#8DA101","#F85552","#DFA000","#DF69BA","#35A77C","#939F91"],
["GithubDark","github","GithubDark","dark","default","#0D1117","#C9D1D9","#58A6FF","#3FB950","#F85149","#D29922","#DB6D28","#79C0FF","#8B949E"],
["GithubLight","github","GithubLight","light","default","#FFFFFF","#24292F","#0969DA","#1A7F37","#CF222E","#9A6700","#BC4C00","#218BFF","#6E7781"],
["GithubDarkDimmed","github","GithubDarkDimmed","dark","dimmed","#22272E","#ADBAC7","#539BF5","#57AB5A","#F47067","#C69026","#F47067","#6CB6FF","#768390"],
["GithubDarkHighContrast","github","GithubDarkHighContrast","dark","high-contrast-dark","#0A0C10","#F0F3F6","#71B7FF","#26CD4D","#FF6A69","#F0B72F","#FF6A69","#91CBFF","#9198A1"],
["GithubLightHighContrast","github","GithubLightHighContrast","light","high-contrast-light","#FFFFFF","#0E1116","#1A69DB","#104F24","#A0111F","#7D4E00","#A0111F","#034188","#69717B"],
["GruvboxDark","gruvbox","GruvboxDark","dark","default","#282828","#D5C4A1","#D79921","#98971A","#CC241D","#D79921","#D65D0E","#689D6A","#928374"],
["GruvboxLight","gruvbox","GruvboxLight","light","default","#FBF1C7","#3C3836","#D65D0E","#79740E","#CC241D","#B57614","#D65D0E","#076678","#928374"],
["Houston","houston","Houston","dark","solo","#17191E","#CDD6F4","#F9C86A","#4AF2C8","#FF5370","#FFA726","#81D4FA","#4AF2C8","#545878"],
["KanagawaWave","kanagawa","KanagawaWave","dark","default","#1F1F28","#DCD7BA","#7E9CD8","#98BB6C","#C34043","#FF9E3B","#D27E99","#7AA89F","#727169"],
["KanagawaDragon","kanagawa","KanagawaDragon","dark","dragon","#181616","#C5C9C5","#7EB3C9","#87A987","#C4746E","#B6927B","#C4746E","#8EA4A2","#8A8980"],
["KanagawaLotus","kanagawa","KanagawaLotus","light","default","#F2ECBC","#545464","#1F5F8A","#4E7C3F","#C84053","#835C00","#B5485D","#536A5B","#A09F8F"],
["Laserwave","laserwave","Laserwave","dark","solo","#27212E","#FFFFFF","#EB64B9","#74DFC4","#FE4450","#FFEE79","#FFEE79","#74DFC4","#6B5F7D"],
["Material","material","Material","dark","default","#263238","#EEFFFF","#82AAFF","#C3E88D","#F07178","#FFCB6B","#F78C6C","#89DDFF","#546E7A"],
["MaterialOcean","material","MaterialOcean","dark","ocean","#0F111A","#A6ACCD","#82AAFF","#C3E88D","#F07178","#FFCB6B","#F78C6C","#89DDFF","#464B5D"],
["MaterialPalenight","material","MaterialPalenight","dark","palenight","#292D3E","#A6ACCD","#82AAFF","#C3E88D","#F07178","#FFCB6B","#F78C6C","#89DDFF","#676E95"],
["MaterialLighter","material","MaterialLighter","light","default","#FAFAFA","#546E7A","#6182B8","#91B859","#E53935","#F6A434","#F76D47","#39ADB5","#90A4AE"],
["MaterialDarker","material","MaterialDarker","dark","darker","#212121","#EEFFFF","#89DDFF","#C3E88D","#FF5370","#FFCB6B","#F78C6C","#82AAFF","#546E7A"],
["MinDark","min","MinDark","dark","default","#1F1F1F","#B2B2B2","#569CD6","#B5CEA8","#F44747","#CCA700","#CE9178","#4EC9B0","#525252"],
["MinLight","min","MinLight","light","default","#F8F8F8","#333333","#0000FF","#098658","#E50000","#865F00","#C1440E","#267F99","#9A9A9A"],
["Monokai","monokai","Monokai","dark","solo","#272822","#F8F8F2","#F92672","#A6E22E","#F92672","#E6DB74","#AE81FF","#66D9EF","#75715E"],
["Moonlight","moonlight","Moonlight","dark","solo","#212337","#C8D3F5","#82AAFF","#C3E88D","#FF757F","#FFC777","#F78C6C","#86E1FC","#7A88CF"],
["NightOwl","night-owl","NightOwl","dark","default","#011627","#D6DEEB","#82AAFF","#22DA6E","#EF5350","#C5E478","#F78C6C","#21C7A8","#637777"],
["NightOwlLight","night-owl","NightOwlLight","light","default","#FBFBFB","#403F53","#4876D6","#2AA298","#D3423E","#DAA520","#DD6A58","#08916A","#989FB1"],
["Nord","nord","Nord","dark","solo","#2E3440","#D8DEE9","#88C0D0","#A3BE8C","#BF616A","#EBCB8B","#D08770","#81A1C1","#4C566A"],
["OneDarkPro","one-dark","OneDarkPro","dark","default","#282C34","#ABB2BF","#61AFEF","#98C379","#E06C75","#E5C07B","#D19A66","#56B6C2","#5C6370"],
["OneLight","one-dark","OneLight","light","default","#FAFAFA","#383A42","#4078F2","#50A14F","#CA1243","#C18401","#986801","#0184BC","#A0A1A7"],
["Plastic","plastic","Plastic","dark","solo","#1B1D23","#ABB2BF","#61AFEF","#98C379","#E06C75","#E5C07B","#D19A66","#56B6C2","#7A7E8A"],
["Poimandres","poimandres","Poimandres","dark","solo","#1B1E28","#A6ACCD","#A6DA95","#5DE4C7","#D0679D","#FFFAC2","#D0679D","#ADD7FF","#767C9D"],
["Projectious","projectious","Projectious","dark","solo","#0E1720","#C5DAF0","#E05232","#4FB07A","#E55B5B","#E0B85B","#F2A65A","#8AACC8","#7B8DA3"],
["Red","red","Red","dark","solo","#390000","#F8F8F8","#FF6666","#F4C2C2","#FF0000","#FF8800","#FFD0D0","#FF9999","#A06060"],
["RosePine","rose-pine","RosePine","dark","default","#191724","#E0DEF4","#C4A7E7","#31748F","#EB6F92","#F6C177","#EA9A97","#9CCFD8","#6E6A86"],
["RosePineMoon","rose-pine","RosePineMoon","dark","moon","#232136","#E0DEF4","#C4A7E7","#3E8FB0","#EB6F92","#F6C177","#EA9A97","#9CCFD8","#6E6A86"],
["RosePineDawn","rose-pine","RosePineDawn","light","default","#FAF4ED","#575279","#907AA9","#56949F","#B4637A","#EA9D34","#D7827E","#286983","#9893A5"],
["SlackDark","slack","SlackDark","dark","default","#222529","#D1D2D3","#8CC4FF","#AFE3A4","#E07070","#DFC55A","#DFC55A","#98D1E0","#60656A"],
["SlackOchin","slack","SlackOchin","light","ochin","#F9F9F9","#383A3C","#0070D1","#268829","#D0104C","#C64B10","#C64B10","#007A7A","#A0A4A8"],
["SnazzyLight","snazzy","SnazzyLight","light","solo","#FAFBFC","#2D2D2D","#57C7FF","#5AF78E","#FF5C57","#FF9F43","#FF6AC1","#57C7FF","#9E9E9E"],
["SolarizedDark","solarized","SolarizedDark","dark","default","#002B36","#93A1A1","#268BD2","#859900","#DC322F","#B58900","#CB4B16","#2AA198","#657B83"],
["SolarizedLight","solarized","SolarizedLight","light","default","#FDF6E3","#586E75","#268BD2","#859900","#DC322F","#B58900","#CB4B16","#2AA198","#93A1A1"],
["Synthwave84","synthwave-84","Synthwave84","dark","solo","#2A2139","#FFFFFF","#36F9F6","#FF7EDB","#FE4450","#FEDE5D","#F97E72","#36F9F6","#848082"],
["TokyoNight","tokyo-night","TokyoNight","dark","default","#1A1B26","#C0CAF5","#7AA2F7","#9ECE6A","#F7768E","#E0AF68","#FF9E64","#7DCFFF","#565F89"],
["TokyoNightStorm","tokyo-night","TokyoNightStorm","dark","storm","#24283B","#C0CAF5","#7AA2F7","#9ECE6A","#F7768E","#E0AF68","#FF9E64","#7DCFFF","#565F89"],
["TokyoNightDay","tokyo-night","TokyoNightDay","light","default","#E1E2E7","#3760BF","#2E7DE9","#587539","#F52A65","#8C6C3E","#B15C00","#007197","#7B8496"],
["Vesper","vesper","Vesper","dark","solo","#101010","#FFFFFF","#FF7B00","#99FFE4","#F44747","#FF7B00","#FFC799","#FFC799","#5C5C5C"],
["VitesseDark","vitesse","VitesseDark","dark","default","#121212","#DBD7CA","#4D9375","#C98A7D","#E06C75","#D4976C","#6496C8","#80A0C0","#758575"],
["VitesseLight","vitesse","VitesseLight","light","default","#FFFFFF","#393A34","#1E754F","#B56959","#AB5959","#B07D48","#296AA3","#2E808F","#A0A077"],
["VitesseBlack","vitesse","VitesseBlack","dark","black","#000000","#DBD7CA","#4D9375","#C98A7D","#E06C75","#D4976C","#6496C8","#80A0C0","#606060"],
["VsCodeDarkPlus","vscode","VsCodeDarkPlus","dark","default","#1E1E1E","#D4D4D4","#569CD6","#B5CEA8","#F44747","#CCA700","#CE9178","#4EC9B0","#6A9955"],
["VsCodeLightPlus","vscode","VsCodeLightPlus","light","default","#FFFFFF","#000000","#0000FF","#098658","#CD3131","#A65E00","#A31515","#267F99","#008000"]
];

export const VARIANTS = C.map(r => ({
  id: r[0], family: r[1], name: r[2], mode: r[3], variantKey: r[4],
  cur: { bg: r[5], fg: r[6], accent: r[7], green: r[8], red: r[9], yellow: r[10], orange: r[11], cyan: r[12], muted: r[13] }
}));

/* ── Evidence: where a canonical value was read from ─────────────────── */
export const EVIDENCE = {
  kana:  "rebelot/kanagawa.nvim@master · lua/kanagawa/colors.lua (read 2026-08-20)",
  ever:  "sainnhe/everforest@master · palette.md (read 2026-08-20)",
  shiki: "shikijs/textmate-grammars-themes@main · packages/tm-themes/themes/*.json (read 2026-08-20)",
  gruv:  "morhetz/gruvbox palette — bright set is the dark-mode set, faded set the light-mode set",
  nord:  "nordtheme/nord — nord0-nord15 slot definitions",
  primer:"primer/primitives — GitHub fgColor scales per theme",
  cat:   "catppuccin/catppuccin — palette steps (overlay/subtext)",
  tn:    "folke/tokyonight.nvim — palette + comment step",
  one:   "atom/one-dark-syntax — palette",
  mono:  "monokai/monokai — original palette",
  sol:   "altercation/solarized — base03..base3 role table",
  no:    "sdras/night-owl-vscode-theme — palette",
  brand: "projectious.work brand v2.1.1 · brand/tokens/variables.css (mirrored in this project)"
};

/* ── CANON: corrections where the current value is not the upstream value.
   slot: [correctValue, evidenceKey, why]                                  */
export const CANON = {
  Andromeeda: {
    green:  ["#96E072", "shiki", "terminal.ansiGreen / string token is #96E072; #89E044 appears nowhere upstream"],
    yellow: ["#FFE66D", "shiki", "terminal.ansiYellow / type token is #FFE66D; #FFCC00 appears nowhere upstream"],
    muted:  ["#A0A1A7", "shiki", "comment token is #A0A1A7cc; #6B6B6B is invented and fails as text"]
  },
  AuroraX: {
    _suspect: "Every slot equals VS Code Dark+ (#569CD6/#B5CEA8/#F44747/#CE9178/#4EC9B0). Aurora X only shares its background. Treat as an unfilled palette, not a theme."
  },
  EverforestDark: {
    orange: ["#E69875", "ever", "#D699B6 is Everforest *purple*; the orange role is #E69875"],
    muted:  ["#859289", "ever", "grey1 is the comment/UI-text grey; grey0 (#7A8478) is line numbers"]
  },
  EverforestLight: {
    orange: ["#F57D26", "ever", "#DF69BA is Everforest *purple*; the orange role is #F57D26"]
  },
  GithubDark: {
    fg: ["#E6EDF3", "primer", "fgColor.default moved to #e6edf3; #c9d1d9 is the retired 2021 value"]
  },
  GithubDarkDimmed: {
    orange: ["#E0823D", "primer", "orange was duplicating red (#F47067); dimmed fgColor.orange is #e0823d"]
  },
  GithubDarkHighContrast: {
    orange: ["#FFA657", "primer", "orange was duplicating red (#FF6A69)"]
  },
  GithubLightHighContrast: {
    orange: ["#702C00", "primer", "orange was duplicating red (#A0111F)"]
  },
  GruvboxDark: {
    fg:     ["#EBDBB2", "gruv", "fg is fg1 #ebdbb2; #d5c4a1 is fg2"],
    accent: ["#FABD2F", "gruv", "on a dark bg gruvbox uses the *bright* set; neutral #d79921 is the light-mode yellow"],
    green:  ["#B8BB26", "gruv", "bright green — neutral #98971a measures 3.4:1 on #282828"],
    red:    ["#FB4934", "gruv", "bright red — neutral #cc241d measures 2.4:1 on #282828"],
    yellow: ["#FABD2F", "gruv", "bright yellow"],
    orange: ["#FE8019", "gruv", "bright orange"],
    cyan:   ["#8EC07C", "gruv", "bright aqua — neutral #689d6a measures 3.4:1"]
  },
  GruvboxLight: {
    accent: ["#AF3A03", "gruv", "light mode uses the *faded* set; #d65d0e is the neutral orange"],
    red:    ["#9D0006", "gruv", "faded red — neutral #cc241d measures 3.9:1 on #fbf1c7"],
    orange: ["#AF3A03", "gruv", "faded orange"],
    cyan:   ["#427B58", "gruv", "faded aqua; #076678 is the faded *blue*, a different slot"]
  },
  KanagawaWave: {
    red:    ["#E46876", "kana", "waveRed — autumnRed #C34043 is the diff/diag red and measures 3.4:1 on sumiInk3"],
    yellow: ["#E6C384", "kana", "carpYellow is the type/yellow role; #FF9E3B is roninYellow (diagnostic warning)"],
    orange: ["#FFA066", "kana", "surimiOrange — #D27E99 is sakuraPink, a different hue entirely"]
  },
  KanagawaDragon: {
    accent: ["#8BA4B0", "kana", "dragonBlue2 — #7EB3C9 exists nowhere in the kanagawa palette"],
    yellow: ["#C4B28A", "kana", "dragonYellow — #B6927B is dragonOrange"],
    orange: ["#B6927B", "kana", "dragonOrange; orange was duplicating red"],
    muted:  ["#9E9B93", "kana", "dragonGray2 — #8A8980 is lotusGray3, a light-theme value"]
  },
  KanagawaLotus: {
    accent: ["#4D699B", "kana", "lotusBlue4 — #1F5F8A is not in the palette"],
    green:  ["#6F894E", "kana", "lotusGreen — #4E7C3F is not in the palette"],
    yellow: ["#836F4A", "kana", "lotusYellow2 — #835C00 is not in the palette"],
    orange: ["#CC6D00", "kana", "lotusOrange — #B5485D is not in the palette (reads as red)"],
    cyan:   ["#597B75", "kana", "lotusAqua — #536A5B is not in the palette"],
    muted:  ["#716E61", "kana", "lotusGray2 — #A09F8F is not in the palette and measures 2.3:1"]
  },
  Laserwave: {
    red:    ["#FF3E7B", "shiki", "editorError.foreground; #FE4450 is Synthwave '84's red"],
    yellow: ["#FFE261", "shiki", "constant.language token; #FFEE79 is not a LaserWave value"],
    orange: ["#FFB85B", "shiki", "the theme's own orange token — orange was duplicating yellow"],
    cyan:   ["#B4DCE7", "shiki", "terminal.ansiCyan; #74DFC4 is ansiGreen (already the green slot)"],
    muted:  ["#91889B", "shiki", "comment token; #6B5F7D is the punctuation grey"]
  },
  MinDark: {
    fg:     ["#F8F8F8", "shiki", "constant/plain token ink; #B2B2B2 is invented"],
    accent: ["#79B8FF", "shiki", "Min's blue — #569CD6 is VS Code Dark+"],
    red:    ["#F97583", "shiki", "keyword token — #F44747 is VS Code Dark+"],
    yellow: ["#FF9800", "shiki", "parameter token — #CCA700 is VS Code Dark+"],
    orange: ["#FFAB70", "shiki", "string/tag token — #CE9178 is VS Code Dark+"],
    muted:  ["#6B737C", "shiki", "comment token — #525252 is invented"],
    _suspect: "Min has no green and no cyan. Both slots were filled with VS Code Dark+ values; they must be derived and labelled as derived."
  },
  MinLight: {
    _suspect: "Every slot equals VS Code Light+ (#0000FF/#098658/#E50000/#865F00/#267F99). Min Light shares none of them beyond the background."
  },
  Monokai: {
    orange: ["#FD971F", "mono", "Monokai's orange; #AE81FF is its purple, which belongs in the new magenta slot"]
  },
  NightOwl: {
    yellow: ["#ECC48D", "no", "strings/yellow role; #C5E478 is the green-yellow already covered by green"]
  },
  Nord: {
    cyan: ["#8FBCBB", "nord", "nord7 is cyan; #81A1C1 is nord9, the blue"]
  },
  Vesper: {
    accent: ["#FFC799", "shiki", "the theme's single accent (button/focus/tab border); #FF7B00 is not a Vesper value"],
    red:    ["#FF8080", "shiki", "editorError.foreground — #F44747 is VS Code Dark+"],
    yellow: ["#FFC799", "shiki", "editorWarning.foreground — Vesper has one warm hue by design"],
    cyan:   ["#99FFE4", "shiki", "the mint/aqua; cyan was duplicating orange"],
    muted:  ["#A0A0A0", "shiki", "keyword/UI grey; #5C5C5C is below the comment token's effective ink"]
  },
  Red: {
    _suspect: "green #F4C2C2 and orange #FFD0D0 are pale pinks, not a green or an orange. Semantic status colour is unreadable as status."
  },
  DraculaSoft: {
    _suspect: "Unverified against dracula-soft.json; bg #22212C matches Dracula Pro, not Dracula Soft."
  }
};

/* ── In-palette lift targets. When a slot fails its contrast floor, prefer a
   step that already exists in the upstream palette over a computed colour. */
export const LIFTS = {
  CatppuccinMocha:      { muted: ["#9399B2", "overlay2", "cat"] },
  CatppuccinMacchiato:  { muted: ["#939AB7", "overlay2", "cat"] },
  CatppuccinFrappe:     { muted: ["#949CBB", "overlay2", "cat"] },
  CatppuccinLatte:      { muted: ["#6C6F85", "subtext0", "cat"] },
  TokyoNight:           { muted: ["#7982A9", "comment (lifted step)", "tn"] },
  TokyoNightStorm:      { muted: ["#7982A9", "comment (lifted step)", "tn"] },
  OneDarkPro:           { muted: ["#7F848E", "comment grey", "one"] },
  Nord:                 { muted: ["#7B88A1", "nord3 lifted (community comment step)", "nord"] },
  GruvboxDark:          { muted: ["#A89984", "gray/light4", "gruv"] },
  GruvboxLight:         { muted: ["#7C6F64", "dark4", "gruv"] },
  SolarizedLight:       { muted: ["#93A1A1", "base1 — kept for chrome, text must use base01", "sol"] },
  RosePine:             { muted: ["#908CAA", "subtle", "shiki"] },
  RosePineMoon:         { muted: ["#908CAA", "subtle", "shiki"] },
  RosePineDawn:         { muted: ["#797593", "subtle", "shiki"] },
  EverforestLight:      { muted: ["#829181", "grey2", "ever"] },
  KanagawaWave:         { muted: ["#938AA9", "springViolet1", "kana"] },
  Material:             { muted: ["#697C85", "comment step", "shiki"] },
  MaterialOcean:        { muted: ["#5C6379", "comment step", "shiki"] },
  Houston:              { muted: ["#6E739A", "comment step", "shiki"] }
};

/* ── New first-class slots that the nine-slot palette cannot express.
   Canonical magenta/purple where the upstream theme has one.             */
export const MAGENTA = {
  Andromeeda: "#C74DED", Dracula: "#FF79C6", DraculaSoft: "#FF79C6", Monokai: "#AE81FF",
  Nord: "#B48EAD", KanagawaWave: "#957FB8", KanagawaDragon: "#A292A3", KanagawaLotus: "#B35B79",
  EverforestDark: "#D699B6", EverforestLight: "#DF69BA", Laserwave: "#B381C5",
  TokyoNight: "#BB9AF7", TokyoNightStorm: "#BB9AF7", TokyoNightDay: "#9854F1",
  CatppuccinMocha: "#CBA6F7", CatppuccinMacchiato: "#C6A0F6", CatppuccinFrappe: "#CA9EE6", CatppuccinLatte: "#8839EF",
  OneDarkPro: "#C678DD", OneLight: "#A626A4", Plastic: "#C678DD",
  Material: "#C792EA", MaterialOcean: "#C792EA", MaterialPalenight: "#C792EA", MaterialDarker: "#C792EA", MaterialLighter: "#7C4DFF",
  NightOwl: "#C792EA", NightOwlLight: "#994CC3", RosePine: "#C4A7E7", RosePineMoon: "#C4A7E7", RosePineDawn: "#907AA9",
  SolarizedDark: "#D33682", SolarizedLight: "#D33682", GithubDark: "#D2A8FF", GithubLight: "#8250DF",
  GithubDarkDimmed: "#DCBDFB", GithubDarkHighContrast: "#DBB7FF", GithubLightHighContrast: "#5A1E96",
  AyuDark: "#D2A6FF", AyuMirage: "#D4BFFF", AyuLight: "#A37ACC", GruvboxDark: "#D3869B", GruvboxLight: "#8F3F71",
  Moonlight: "#C099FF", Poimandres: "#FCC5E9", Synthwave84: "#FF7EDB", MinDark: "#B392F0",
  VitesseDark: "#CB7676", VitesseLight: "#B56959", VitesseBlack: "#CB7676", Houston: "#FF6BCB",
  Vesper: "#FFC799", Projectious: "#D491B4", Red: "#FF99CC", SlackDark: "#D6A6E0", SlackOchin: "#8E4EC6",
  SnazzyLight: "#FF6AC1", VsCodeDarkPlus: "#C586C0", VsCodeLightPlus: "#AF00DB", MaterialLighterX: "#7C4DFF",
  AuroraX: "#C586C0", MinLight: "#AF00DB"
};

/* ── projectious — five variants, built from brand v2.1.1 tokens ─────── */
export const PROJECTIOUS = [
  { id: "ProjectiousNavy", name: "Projectious · navy dark", mode: "dark", variantKey: "default (navy)",
    note: "Brand default dark. Page is midnight-1 dark (#132440); code panels stay on #0e1720 so a code block still reads as an inset panel.",
    p: { bg:"#132440", fg:"#C5DAF0", accent:"#E05232", green:"#6CC090", red:"#F08B80", yellow:"#E0A92A", orange:"#EA7558", cyan:"#74C0C9", muted:"#97A8B8" },
    x: { surface:"#1A2B3E", magenta:"#D491B4", border_inactive:"#7B8DA3", cursor:"#E05232", cursor_text:"#0E1720",
         selection_bg:"#3A5C7E", selection_fg:"#C5DAF0", accent_text:"#EA7558", code_panel:"#0E1720", terminal:"#0E1720" } },
  { id: "ProjectiousDeep", name: "Projectious · deep dark", mode: "dark", variantKey: "deep",
    note: "The bottom of the midnight ramp. Same ink, deeper page — panels lift by surface step, not shadow.",
    p: { bg:"#0E1720", fg:"#C5DAF0", accent:"#E05232", green:"#6CC090", red:"#F08B80", yellow:"#E0A92A", orange:"#EA7558", cyan:"#74C0C9", muted:"#97A8B8" },
    x: { surface:"#131E2B", magenta:"#D491B4", border_inactive:"#7B8DA3", cursor:"#E05232", cursor_text:"#0E1720",
         selection_bg:"#2E4B68", selection_fg:"#C5DAF0", accent_text:"#EA7558", code_panel:"#131E2B", terminal:"#0E1720" } },
  { id: "ProjectiousLight", name: "Projectious · light", mode: "light", variantKey: "default",
    note: "Page is midnight-1 (#f8f9fb), not white; white returns as the raised surface. Accent text takes orange-11, never orange-9.",
    p: { bg:"#F8F9FB", fg:"#142438", accent:"#C04424", green:"#276754", red:"#A8261C", yellow:"#8B6508", orange:"#C94208", cyan:"#1C6B6B", muted:"#5C6F82" },
    x: { surface:"#FFFFFF", magenta:"#8A3F6E", border_inactive:"#ADB2BA", cursor:"#E05232", cursor_text:"#F4F5F7",
         selection_bg:"#C3D1E3", selection_fg:"#142438", accent_fill:"#CC4528", code_panel:"#0E1720", terminal:"#0E1720" } },
  { id: "ProjectiousHCDark", name: "Projectious · high-contrast dark", mode: "dark", variantKey: "high-contrast-dark",
    note: "data-contrast=high, dark. Text roles pushed to the ends of the ramp; accent lifted to #ff8161 so it clears AA as text.",
    p: { bg:"#0E1720", fg:"#FFFFFF", accent:"#FF8161", green:"#A8E6C4", red:"#FFC0B8", yellow:"#FFDF94", orange:"#FFB49C", cyan:"#9BD6DD", muted:"#C5DAF0" },
    x: { surface:"#1A2B3E", magenta:"#F0B6D3", border_inactive:"#C5DAF0", cursor:"#FF8161", cursor_text:"#0E1720",
         selection_bg:"#3A5C7E", selection_fg:"#FFFFFF", code_panel:"#0E1720", terminal:"#0E1720" } },
  { id: "ProjectiousHCLight", name: "Projectious · high-contrast light", mode: "light", variantKey: "high-contrast-light",
    note: "data-contrast=high, light. Tints flatten to solid borders; every text role sits at the dark end of its scale.",
    p: { bg:"#FFFFFF", fg:"#000000", accent:"#A02F16", green:"#0F4D3A", red:"#7A1610", yellow:"#4A3400", orange:"#8A2E0A", cyan:"#0F4C4C", muted:"#1E2B38" },
    x: { surface:"#F8F9FB", magenta:"#6E1A50", border_inactive:"#1E2B38", cursor:"#A02F16", cursor_text:"#FFFFFF",
         selection_bg:"#B0C1D6", selection_fg:"#000000", code_panel:"#0E1720", terminal:"#0E1720" } }
];

/* ── Contrast math (WCAG 2.1 relative luminance) ─────────────────────── */
export const rgb = h => { h = h.replace("#",""); if (h.length===3) h = h.split("").map(c=>c+c).join("");
  return [parseInt(h.slice(0,2),16), parseInt(h.slice(2,4),16), parseInt(h.slice(4,6),16)]; };
export const hex = ([r,g,b]) => "#" + [r,g,b].map(v => Math.max(0,Math.min(255,Math.round(v))).toString(16).padStart(2,"0")).join("").toUpperCase();
export const lum = h => { const [r,g,b] = rgb(h).map(v => { v/=255; return v<=0.03928 ? v/12.92 : Math.pow((v+0.055)/1.055, 2.4); });
  return 0.2126*r + 0.7152*g + 0.0722*b; };
export const ratio = (a,b) => { const la = lum(a), lb = lum(b); return (Math.max(la,lb)+0.05)/(Math.min(la,lb)+0.05); };
export const cr = (a,b) => Math.round(ratio(a,b)*100)/100;
export const mix = (a,b,t) => { const A = rgb(a), B = rgb(b); return hex([0,1,2].map(i => A[i] + (B[i]-A[i])*t)); };

/* Lift a colour away from its background until it clears `target`, keeping hue.
   Steps in sRGB toward white (on dark bg) or black (on light bg). */
export function lift(color, bg, target) {
  if (ratio(color, bg) >= target) return color;
  const toWhite = lum(bg) < 0.18;
  const dest = toWhite ? "#FFFFFF" : "#000000";
  let best = color;
  for (let t = 0.02; t <= 1.0001; t += 0.02) {
    best = mix(color, dest, t);
    if (ratio(best, bg) >= target) break;
  }
  return best;
}

/* Per-role contrast floors, with the reason each floor is what it is. */
export const ROLE_TARGETS = {
  fg:      [7.0, "body text, read for hours — AAA for the one colour every character uses"],
  muted:   [4.5, "comments and metadata are text, not decoration — SC 1.4.3 applies"],
  accent:  [4.5, "accent is drawn as text (directories, titles, function names), not only as a fill"],
  green:   [4.5, "syntax + success status, both carrying text"],
  red:     [4.5, "syntax + error status — the one colour a user must never miss"],
  yellow:  [4.5, "syntax types + warning status"],
  orange:  [4.5, "numbers and constants — dense small glyphs"],
  cyan:    [4.5, "operators and info status"],
  magenta: [4.5, "keywords/preprocessor once it becomes first-class"],
  surface: [1.20, "status bars and popups must be *visible* against the page, not readable — a luminance step, not a text pair"],
  pane_inactive_fg: [3.0, "deliberately dim, still legible: SC 1.4.3 large-text floor"],
  border:  [3.0, "non-text UI contrast, SC 1.4.11"]
};

/* Derive a per-theme surface: step the background toward the foreground until
   it is perceptibly distinct, then verify fg still reads on it. */
export function surfaceFor(bg, fg) {
  let s = bg;
  for (let t = 0.02; t <= 0.5; t += 0.01) { s = mix(bg, fg, t); if (ratio(s, bg) >= 1.20) break; }
  return s;
}
/* A selection is a short inline run inside a page of the same colour, so the
   1.20 status-bar step is not enough. Floors: 1.8 dark, 1.4 light. */
export const SELECTION_FLOOR = { dark: 1.8, light: 1.4 };
export function selectionFor(bg, accent, mode) {
  const floor = SELECTION_FLOOR[mode] || 1.8;
  let s = mix(bg, accent, mode === "light" ? 0.14 : 0.22);
  for (let t = mode === "light" ? 0.14 : 0.22; t <= 0.9; t += 0.02) {
    s = mix(bg, accent, t);
    if (ratio(s, bg) >= floor) break;
  }
  return s;
}
export function tintFor(bg, hue, t) { return mix(bg, hue, t); }
/* Pick the readable ink for a solid fill from the theme's own colours. */
export function inkFor(fill, bg, fg) {
  const cands = [bg, fg, "#FFFFFF", "#000000"];
  return cands.reduce((a,b) => ratio(b,fill) > ratio(a,fill) ? b : a);
}

/* Build the corrected, contrast-checked palette for one variant. */
export function resolve(v) {
  const canon = CANON[v.id] || {}, lifts = LIFTS[v.id] || {};
  const out = {}, notes = [];
  for (const s of SLOTS) {
    const cur = v.cur[s];
    let val = cur, kind = "kept", why = "", ev = "";
    if (canon[s]) { val = canon[s][0]; ev = canon[s][1]; why = canon[s][2]; kind = "canon"; }
    const bg = s === "bg" ? null : (canon.bg ? canon.bg[0] : v.cur.bg);
    if (bg && s !== "bg") {
      const target = (ROLE_TARGETS[s] || [4.5])[0];
      if (ratio(val, bg) < target) {
        if (lifts[s]) {
          const alt = lifts[s][0];
          if (ratio(alt, bg) >= target - 0.15) {
            val = alt; kind = kind === "canon" ? "canon+step" : "step";
            why = (why ? why + " " : "") + `Lifted to the in-palette step ${lifts[s][1]} for ${target}:1.`;
            ev = ev || lifts[s][2];
          }
        }
        if (ratio(val, bg) < target) {
          const l = lift(val, bg, target);
          if (l !== val) {
            why = (why ? why + " " : "") + `Computed lift ${val}→${l}: ${cr(val,bg)}:1 misses the ${target}:1 floor.`;
            val = l; kind = kind.startsWith("canon") ? "canon+lift" : "lift";
          }
        }
      }
    }
    out[s] = { cur, val, kind, why, ev, ratio: s === "bg" ? null : cr(val, canon.bg ? canon.bg[0] : v.cur.bg),
               curRatio: s === "bg" ? null : cr(cur, v.cur.bg) };
  }
  const bg = out.bg.val, fg = out.fg.val;
  const ext = {
    magenta: MAGENTA[v.id] ? { val: lift(MAGENTA[v.id], bg, 4.5), src: "upstream palette" }
                           : { val: lift(mix(out.red.val, out.accent.val, 0.4), bg, 4.5), src: "derived (no upstream magenta)" },
    surface: { val: surfaceFor(bg, fg), src: "derived from this theme's own bg/fg" },
    selection_bg: { val: selectionFor(bg, out.accent.val, v.mode), src: "bg + accent tint, at the selection floor" },
    diff_add_bg: { val: mix(bg, out.green.val, v.mode === "light" ? 0.16 : 0.22), src: "bg + green tint" },
    diff_del_bg: { val: mix(bg, out.red.val, v.mode === "light" ? 0.16 : 0.22), src: "bg + red tint" },
    diff_change_bg: { val: mix(bg, out.yellow.val, v.mode === "light" ? 0.16 : 0.22), src: "bg + yellow tint" },
    cursor: { val: out.accent.val, src: "accent" },
    cursor_text: { val: inkFor(out.accent.val, bg, fg), src: "readable ink on the cursor fill" }
  };
  ext.selection_fg = { val: inkFor(ext.selection_bg.val, bg, fg), src: "readable ink on the selection tint" };
  ext.border_active = { val: lift(out.accent.val, bg, 3.0), src: "accent at the 3:1 non-text floor" };
  ext.border_inactive = { val: lift(out.muted.val, bg, 3.0), src: "muted at the 3:1 non-text floor" };
  ext.pane_inactive_bg = { val: mix(bg, out.muted.val, 0.12), src: "existing derivation, kept" };
  ext.pane_inactive_fg = { val: lift(mix(fg, out.muted.val, 0.25), ext.pane_inactive_bg.val, 3.0), src: "existing derivation + 3:1 floor" };
  ext.status_active_ink = { val: inkFor(out.accent.val, bg, fg), src: "computed ink on the accent fill" };
  return { out, ext, notes, suspect: canon._suspect || null };
}

/* ── New families authored for this audit ─────────────────────────────────
   mono          grayscale, one dark + one light. No hue at all, so role
                 separation is carried by a lightness step PLUS an attribute.
   contrast      colour, high (7:1) and max (12:1) levels, dark + light.
   contrast-mono grayscale at the same two levels — the hardest case, where
                 the usable ramp shrinks to three steps and typography does
                 nearly all the work.
   Every role carries {v: value, a: attribute}. Floors are enforced by the
   verifier below, so no authored value can quietly miss its target.         */
export const NEWFAMS = [
  { id: "MonoDark", family: "mono", name: "Mono · dark", mode: "dark", variantKey: "solo",
    floor: 4.5, emphasisMin: "standard",
    note: "A working grayscale theme, not an accessibility mode. Six roles sit on four lightness steps; the attribute disambiguates same-step pairs (orange plain vs red bold-underline, green plain vs cyan italic).",
    p: { bg:{v:"#121212",a:""}, fg:{v:"#EDEDED",a:""}, accent:{v:"#FFFFFF",a:"bold"},
         green:{v:"#BDBDBD",a:""}, red:{v:"#EDEDED",a:"bold underline"}, yellow:{v:"#D6D6D6",a:"bold"},
         orange:{v:"#D6D6D6",a:""}, cyan:{v:"#BDBDBD",a:"italic"}, muted:{v:"#8A8A8A",a:"italic"} },
    x: { magenta:{v:"#A3A3A3",a:"bold"}, surface:"#262626", selection_bg:"#3D3D3D", selection_fg:"#FFFFFF",
         cursor:"#FFFFFF", cursor_text:"#121212", border_active:"#FFFFFF", border_inactive:"#757575",
         diff_add_bg:"#1E1E1E", diff_del_bg:"#2B2B2B", diff_change_bg:"#242424" } },

  { id: "MonoLight", family: "mono", name: "Mono · light", mode: "light", variantKey: "solo",
    floor: 4.5, emphasisMin: "standard",
    note: "The dark ladder inverted: darkest step plus bold is the accent, and the page stays off-white so a white surface can still lift off it.",
    p: { bg:{v:"#FAFAFA",a:""}, fg:{v:"#1A1A1A",a:""}, accent:{v:"#000000",a:"bold"},
         green:{v:"#545454",a:""}, red:{v:"#1A1A1A",a:"bold underline"}, yellow:{v:"#383838",a:"bold"},
         orange:{v:"#383838",a:""}, cyan:{v:"#545454",a:"italic"}, muted:{v:"#6B6B6B",a:"italic"} },
    x: { magenta:{v:"#454545",a:"bold"}, surface:"#E4E4E4", selection_bg:"#DEDEDE", selection_fg:"#000000",
         cursor:"#000000", cursor_text:"#FAFAFA", border_active:"#000000", border_inactive:"#8A8A8A",
         diff_add_bg:"#EFEFEF", diff_del_bg:"#E2E2E2", diff_change_bg:"#E9E9E9" } },

  { id: "ContrastDark", family: "contrast", name: "Contrast · dark", mode: "dark", variantKey: "high",
    floor: 7.0, emphasisMin: "standard",
    note: "AAA for every text role, hues kept far apart in hue angle rather than merely light. Yellow and orange are the closest pair, so orange stays plain and yellow takes bold.",
    p: { bg:{v:"#0B0B0B",a:""}, fg:{v:"#F5F5F5",a:""}, accent:{v:"#7FB8FF",a:"bold"},
         green:{v:"#5FD98A",a:""}, red:{v:"#FF8A80",a:"bold"}, yellow:{v:"#F2D65C",a:"bold"},
         orange:{v:"#FFAB5E",a:""}, cyan:{v:"#6FE0E0",a:"italic"}, muted:{v:"#B0B8C0",a:"italic"} },
    x: { magenta:{v:"#F09AE0",a:"bold"}, surface:"#242424", selection_bg:"#2E3A46", selection_fg:"#FFFFFF",
         cursor:"#7FB8FF", cursor_text:"#0B0B0B", border_active:"#7FB8FF", border_inactive:"#B0B8C0",
         diff_add_bg:"#0F2416", diff_del_bg:"#2A1210", diff_change_bg:"#241E0C" } },

  { id: "ContrastDarkMax", family: "contrast", name: "Contrast · dark, extreme", mode: "dark", variantKey: "max",
    floor: 12.0, emphasisMin: "full",
    note: "Pure black page, every text role at 12:1 or better. Hues survive because they are pushed toward their light end rather than desaturated; underline returns as a third channel for error and current match.",
    p: { bg:{v:"#000000",a:""}, fg:{v:"#FFFFFF",a:""}, accent:{v:"#A8D4FF",a:"bold"},
         green:{v:"#7CF0A8",a:""}, red:{v:"#FFB3AC",a:"bold underline"}, yellow:{v:"#FFE066",a:"bold"},
         orange:{v:"#FFC98A",a:""}, cyan:{v:"#8FF5F5",a:"italic"}, muted:{v:"#D0D0D0",a:"italic"} },
    x: { magenta:{v:"#FFB3F0",a:"bold"}, surface:"#1A1A1A", selection_bg:"#33404D", selection_fg:"#FFFFFF",
         cursor:"#A8D4FF", cursor_text:"#000000", border_active:"#A8D4FF", border_inactive:"#D0D0D0",
         diff_add_bg:"#0A2614", diff_del_bg:"#2B0F0C", diff_change_bg:"#26200A" } },

  { id: "ContrastLight", family: "contrast", name: "Contrast · light", mode: "light", variantKey: "high",
    floor: 7.0, emphasisMin: "standard",
    note: "White page — the one place white is correct, because any tint spends contrast the dark text needs. Hues are the dark end of each ramp, all AAA.",
    p: { bg:{v:"#FFFFFF",a:""}, fg:{v:"#121212",a:""}, accent:{v:"#0B4FA8",a:"bold"},
         green:{v:"#14663D",a:""}, red:{v:"#A3121A",a:"bold"}, yellow:{v:"#6B4C00",a:"bold"},
         orange:{v:"#8A3200",a:""}, cyan:{v:"#0B5C5C",a:"italic"}, muted:{v:"#4A4A4A",a:"italic"} },
    x: { magenta:{v:"#7A0F6B",a:"bold"}, surface:"#E6E6E6", selection_bg:"#CFE0F5", selection_fg:"#0B1A2B",
         cursor:"#0B4FA8", cursor_text:"#FFFFFF", border_active:"#0B4FA8", border_inactive:"#4A4A4A",
         diff_add_bg:"#E4F2E8", diff_del_bg:"#F7E4E4", diff_change_bg:"#F5EEDC" } },

  { id: "ContrastLightMax", family: "contrast", name: "Contrast · light, extreme", mode: "light", variantKey: "max",
    floor: 12.0, emphasisMin: "full",
    note: "Black text on white, hues at 12:1 or better. At this level hue is nearly exhausted as a channel — the darkest blue and the darkest cyan differ by little — so attributes are mandatory, not optional.",
    p: { bg:{v:"#FFFFFF",a:""}, fg:{v:"#000000",a:""}, accent:{v:"#002B6B",a:"bold"},
         green:{v:"#003D22",a:""}, red:{v:"#6B0008",a:"bold underline"}, yellow:{v:"#3D2A00",a:"bold"},
         orange:{v:"#4F1A00",a:""}, cyan:{v:"#00363A",a:"italic"}, muted:{v:"#262626",a:"italic"} },
    x: { magenta:{v:"#4A0040",a:"bold"}, surface:"#E0E0E0", selection_bg:"#C2D4EB", selection_fg:"#000000",
         cursor:"#002B6B", cursor_text:"#FFFFFF", border_active:"#002B6B", border_inactive:"#262626",
         diff_add_bg:"#DCEBE2", diff_del_bg:"#F2DCDC", diff_change_bg:"#EDE6D2" } },

  { id: "ContrastMonoDark", family: "contrast-mono", name: "Contrast mono · dark", mode: "dark", variantKey: "high",
    floor: 7.0, emphasisMin: "standard",
    note: "Grayscale at AAA. Five usable steps remain above 7:1, so two roles share a step wherever their attributes differ.",
    p: { bg:{v:"#000000",a:""}, fg:{v:"#F0F0F0",a:""}, accent:{v:"#FFFFFF",a:"bold"},
         green:{v:"#A6A6A6",a:""}, red:{v:"#F0F0F0",a:"bold underline"}, yellow:{v:"#D4D4D4",a:"bold"},
         orange:{v:"#D4D4D4",a:""}, cyan:{v:"#A6A6A6",a:"italic"}, muted:{v:"#8C8C8C",a:"italic"} },
    x: { magenta:{v:"#BFBFBF",a:"bold"}, surface:"#1F1F1F", selection_bg:"#383838", selection_fg:"#FFFFFF",
         cursor:"#FFFFFF", cursor_text:"#000000", border_active:"#FFFFFF", border_inactive:"#8C8C8C",
         diff_add_bg:"#171717", diff_del_bg:"#242424", diff_change_bg:"#1D1D1D" } },

  { id: "ContrastMonoDarkMax", family: "contrast-mono", name: "Contrast mono · dark, extreme", mode: "dark", variantKey: "max",
    floor: 12.0, emphasisMin: "full",
    note: "The honest hard case: above 12:1 on black the ramp holds three steps. Nine roles onto three steps means typography is the primary channel and lightness the secondary one — the inverse of every other theme here. This variant must refuse emphasis = none.",
    p: { bg:{v:"#000000",a:""}, fg:{v:"#E6E6E6",a:""}, accent:{v:"#FFFFFF",a:"bold"},
         green:{v:"#CCCCCC",a:""}, red:{v:"#FFFFFF",a:"bold underline"}, yellow:{v:"#E6E6E6",a:"bold"},
         orange:{v:"#CCCCCC",a:"bold"}, cyan:{v:"#E6E6E6",a:"italic"}, muted:{v:"#CCCCCC",a:"italic"} },
    x: { magenta:{v:"#FFFFFF",a:"italic"}, surface:"#1A1A1A", selection_bg:"#333333", selection_fg:"#FFFFFF",
         cursor:"#FFFFFF", cursor_text:"#000000", border_active:"#FFFFFF", border_inactive:"#CCCCCC",
         diff_add_bg:"#141414", diff_del_bg:"#222222", diff_change_bg:"#1B1B1B" } },

  { id: "ContrastMonoLight", family: "contrast-mono", name: "Contrast mono · light", mode: "light", variantKey: "high",
    floor: 7.0, emphasisMin: "standard",
    note: "Grayscale at AAA on white. The same ladder as the dark variant, inverted.",
    p: { bg:{v:"#FFFFFF",a:""}, fg:{v:"#141414",a:""}, accent:{v:"#000000",a:"bold"},
         green:{v:"#4F4F4F",a:""}, red:{v:"#141414",a:"bold underline"}, yellow:{v:"#333333",a:"bold"},
         orange:{v:"#333333",a:""}, cyan:{v:"#4F4F4F",a:"italic"}, muted:{v:"#616161",a:"italic"} },
    x: { magenta:{v:"#404040",a:"bold"}, surface:"#E6E6E6", selection_bg:"#D6D6D6", selection_fg:"#000000",
         cursor:"#000000", cursor_text:"#FFFFFF", border_active:"#000000", border_inactive:"#616161",
         diff_add_bg:"#EDEDED", diff_del_bg:"#E0E0E0", diff_change_bg:"#E8E8E8" } },

  { id: "ContrastMonoLightMax", family: "contrast-mono", name: "Contrast mono · light, extreme", mode: "light", variantKey: "max",
    floor: 12.0, emphasisMin: "full",
    note: "Three steps again, at the dark end. Print-safe by construction: it survives a photocopier, a projector and greyscale printing unchanged.",
    p: { bg:{v:"#FFFFFF",a:""}, fg:{v:"#1A1A1A",a:""}, accent:{v:"#000000",a:"bold"},
         green:{v:"#333333",a:""}, red:{v:"#000000",a:"bold underline"}, yellow:{v:"#1A1A1A",a:"bold"},
         orange:{v:"#333333",a:"bold"}, cyan:{v:"#1A1A1A",a:"italic"}, muted:{v:"#333333",a:"italic"} },
    x: { magenta:{v:"#000000",a:"italic"}, surface:"#E0E0E0", selection_bg:"#CCCCCC", selection_fg:"#000000",
         cursor:"#000000", cursor_text:"#FFFFFF", border_active:"#000000", border_inactive:"#333333",
         diff_add_bg:"#EAEAEA", diff_del_bg:"#DEDEDE", diff_change_bg:"#E5E5E5" } }
];

/* Verify (and if necessary lift) every authored role to its family floor, and
   report which same-step pairs rely on their attribute to stay distinct. */
export function resolveNew(t) {
  const bg = t.p.bg.v, rows = [], seen = {};
  for (const s of SLOTS) {
    if (s === "bg") { rows.push({ slot: s, v: bg, a: "", ratio: null, lifted: false }); continue; }
    const authored = t.p[s].v;
    const v = lift(authored, bg, t.floor);
    rows.push({ slot: s, v, a: t.p[s].a, ratio: cr(v, bg), lifted: v !== authored, authored });
  }
  const mgv = lift(t.x.magenta.v, bg, t.floor);
  rows.push({ slot: "magenta", v: mgv, a: t.x.magenta.a, ratio: cr(mgv, bg), lifted: mgv !== t.x.magenta.v, authored: t.x.magenta.v });
  const collisions = [];
  for (const r of rows) { if (r.slot === "bg") continue; (seen[r.v] = seen[r.v] || []).push(r); }
  for (const k of Object.keys(seen)) if (seen[k].length > 1)
    collisions.push({ value: k, roles: seen[k].map(function(r){ return r.slot + " (" + (r.a || "plain") + ")"; }) });
  return { rows, collisions, surfaceRatio: cr(t.x.surface, bg), selectionRatio: cr(t.x.selection_fg, t.x.selection_bg) };
}
