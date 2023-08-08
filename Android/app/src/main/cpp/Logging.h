#ifndef MICROPHONE_LOGGING_H
#define MICROPHONE_LOGGING_H

#include <android/log.h>

#define LOG_TAG "AndroidMicCPP"

#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)

#endif //MICROPHONE_LOGGING_H
