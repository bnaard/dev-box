<?xml version="1.0" encoding="utf-8"?>
<!-- Makes a feed URL readable when opened in a browser. Styling is inline
     because the XSL result is not processed by Hugo's asset pipeline. -->
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="html" encoding="utf-8" indent="yes"/>
  <xsl:template match="/rss/channel">
    <html lang="en">
      <head>
        <meta charset="utf-8"/>
        <meta name="viewport" content="width=device-width, initial-scale=1"/>
        <title><xsl:value-of select="title"/> — feed</title>
        <style>
          body{margin:0;background:#f8f9fb;color:#142438;font:16px/1.6 "Source Sans 3",ui-sans-serif,system-ui,sans-serif}
          .shell{max-width:760px;margin:0 auto;padding:64px 24px 96px}
          .overline{font-size:11px;letter-spacing:.14em;text-transform:uppercase;color:#546a82;margin-bottom:14px}
          h1{font-family:"Plus Jakarta Sans",ui-sans-serif,sans-serif;font-weight:800;font-size:40px;letter-spacing:-.5px;margin:0 0 12px}
          .lede{font-size:18px;color:#546a82;margin:0 0 8px}
          .note{font-size:13.5px;color:#546a82;background:#fff;border:1px solid #cdd0d5;border-radius:9px;padding:12px 16px;margin:24px 0}
          .note code{font-family:"IBM Plex Mono",ui-monospace,monospace;font-size:.9em}
          article{padding:22px 0;border-top:1px solid #cdd0d5}
          .meta{font-family:"IBM Plex Mono",ui-monospace,monospace;font-size:12px;color:#546a82;margin-bottom:6px}
          h2{font-family:"Plus Jakarta Sans",ui-sans-serif,sans-serif;font-weight:700;font-size:20px;letter-spacing:-.4px;margin:0 0 6px}
          h2 a{color:#142438;text-decoration:none}
          h2 a:hover{color:#c04424}
          p.desc{margin:0;color:#546a82;font-size:15px}
        </style>
      </head>
      <body>
        <div class="shell">
          <div class="overline">RSS feed</div>
          <h1><xsl:value-of select="title"/></h1>
          <p class="lede"><xsl:value-of select="description"/></p>
          <div class="note">This is a feed. Paste this page's address into a feed reader to subscribe, or <a href="{link}">visit the site</a>.</div>
          <xsl:for-each select="item">
            <article>
              <div class="meta"><xsl:value-of select="pubDate"/></div>
              <h2><a href="{link}"><xsl:value-of select="title"/></a></h2>
              <p class="desc"><xsl:value-of select="description"/></p>
            </article>
          </xsl:for-each>
        </div>
      </body>
    </html>
  </xsl:template>
</xsl:stylesheet>
