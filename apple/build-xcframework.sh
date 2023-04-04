#!/bin/bash
set -ex

for sdk in iphoneos macosx; do
  echo "Building for $sdk"

  xcodebuild archive \
    -scheme Connlib \
    -sdk $sdk \
    -archivePath ./connlib-$sdk \
    SKIP_INSTALL=NO \
    BUILD_LIBRARY_FOR_DISTRIBUTION=YES
done

xcodebuild -create-xcframework \
  -framework ./connlib-iphoneos.xcarchive/Products/Library/Frameworks/connlib.framework \
  -framework ./connlib-macosx.xcarchive/Products/Library/Frameworks/connlib.framework \
  -output ./Connlib.xcframework

echo "Build successful. Removing temporary archives"
rm -rf ./connlib-iphoneos.xcarchive
rm -rf ./connlib-macosx.xcarchive
