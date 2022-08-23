cmake_minimum_required(VERSION 3.6.0)
project("microphone")

find_package(oboe REQUIRED CONFIG)

add_library(microphone SHARED jni_bridge.cpp OboeRecorder.cpp)
set_target_properties(microphone PROPERTIES CXX_STANDARD 14)
target_link_libraries(microphone oboe::oboe log)
