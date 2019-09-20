NDK_STANDALONE = $(shell pwd)/NDK
ARCHS_IOS = i386-apple-ios x86_64-apple-ios armv7-apple-ios armv7s-apple-ios aarch64-apple-ios
ARCHS_ANDROID = aarch64-linux-android armv7-linux-androideabi i686-linux-android
LIB=libsecurenative.a

all: ios android

ios: $(ARCHS_IOS)

android: $(ARCHS_ANDROID)

.PHONY: $(ARCHS_IOS)
$(ARCHS_IOS): %:
	cargo build --target $@ --release --lib --tests

aarch64-linux-android:
	PATH=$(PATH):$(NDK_STANDALONE)/arm64/bin \
	CC=$@-gcc \
	CXX=$@-g++ \
	cargo build --target $@ --release --lib --tests

armv7-linux-androideabi:
	PATH=$(PATH):$(NDK_STANDALONE)/arm/bin \
	CC=arm-linux-androideabi-gcc \
	CXX=arm-linux-androideabi-g++ \
	cargo build --target $@ --release --lib --tests

i686-linux-android:
	PATH=$(PATH):$(NDK_STANDALONE)/x86/bin \
	CC=i686-linux-android-gcc \
	CXX=i686-linux-android-g++ \
    cargo build --target $@ --release --lib --tests

#$(LIB): $(ARCHS_IOS)
#	lipo -create -output $@ $(foreach arch,$(ARCHS_IOS),$(wildcard target/$(arch)/release/$(LIB)))
