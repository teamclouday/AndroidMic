cmake_minimum_required(VERSION 3.18.1)
project("microphone")

add_library(microphone SHARED jni_bridge.cpp OboeRecorder.cpp)

set(EXTERNAL_DIR "../../../../external")
add_subdirectory(${EXTERNAL_DIR}/oboe ./oboe)
include_directories(${EXTERNAL_DIR}/oboe/include)

target_link_libraries(microphone oboe log)
