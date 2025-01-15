#!/bin/bash
# When run in the output directory (a directory with folders like
# `observatoryA/` `planetB@lat-long/` etc.) it will take all the rendered
# SVG files and render them into a mp4 video with two seconds between photos
directories=$(find . -maxdepth 1 -mindepth 1 -type d)
lineCount=$(echo "$directories" | wc -l)
if command -v parallel &> /dev/null
then
  #echo "Running with parallel"
  echo "$directories" | xargs -I {} echo 'ffmpeg -y -framerate 0.5 -pattern_type glob -width 1080 -height 1080 -i "{}/*.svg" -c:v libx264 -pix_fmt yuv420p "{}.mp4" > /dev/null 2>&1 ' | parallel --bar
else
  #echo "Running with xargs"
  echo "$directories" | xargs -I {} -P 4 sh -c 'ffmpeg -y -framerate 0.5 -pattern_type glob -width 1080 -height 1080 -i "{}/*.svg" -c:v libx264 -pix_fmt yuv420p "{}.mp4" 2>/dev/null'
fi
