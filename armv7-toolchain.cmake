set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_SYSTEM_PROCESSOR armv7)
set(CMAKE_C_COMPILER arm-linux-gnueabihf-gcc)
set(CMAKE_CXX_COMPILER arm-linux-gnueabihf-g++)
# Unsure if this is needed
set(ENV{PKG_CONFIG} arm-linux-gnueabihf-pkg-config)

# Make sure our built libs go somewhere we can find them:
# AFAICT this gets overwritten, but I'll leave it here in case we can figure out why
# set(CMAKE_INSTALL_PREFIX /usr/arm-linux-gnueabihf/)
