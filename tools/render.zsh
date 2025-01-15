#!/bin/bash
# When run in the output directory (a directory with folders like
# `observatoryA/` `planetB@lat-long/` etc.) it will take all the rendered
# SVG files and render them into a mp4 video with two seconds between photos

for d in */ 
do
  d=${d%/}
  ffmpeg -y -framerate 0.5 -width 1080 -height 1080 -pattern_type glob -i "$d/*.svg" -s 1080x1080 -c:v libx264 -pix_fmt yuv420p "$d.mp4"
done
