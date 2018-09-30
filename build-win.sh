#!/bin/sh

SDL_URL="https://www.libsdl.org/release/SDL2-2.0.8-win32-x64.zip"
TTF_URL="https://www.libsdl.org/projects/SDL_ttf/release/SDL2_ttf-2.0.14-win32-x64.zip"
IMG_URL="https://www.libsdl.org/projects/SDL_image/release/SDL2_image-2.0.3-win32-x64.zip"
VERSION="0.1.0"
NAME="epoch-wars-client-rs"

mkdir -p target/sdl
pushd target/sdl
curl -O "$SDL_URL"
curl -O "$TTF_URL"
curl -O "$IMG_URL"
find -iname '*.zip' -exec unzip -u '{}' \;
popd

cargo build --release --target=x86_64-pc-windows-gnu

mkdir -p "target/$NAME-$VERSION/SDL"
cp "target/x86_64-pc-windows/release/$NAME" "target/$NAME-$VERSION"
find target/sdl -iname '*.dll' -exec cp '{}' "target/$NAME-$VERSION" \;
find target/sdl -iname '*.txt' -exec cp '{}' "target/$NAME-$VERSION/SDL" \;
cp LICENSE "target/$NAME-$VERSION"

zip -r "target/$NAME-$VERSION.zip" "target/$NAME-$VERSION"
