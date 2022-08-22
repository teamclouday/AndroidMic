#ifndef MICROPHONE_LOGGING_H
#define MICROPHONE_LOGGING_H

#include <android/log.h>

#define LOGTAG "AndroidMicCPP"

#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOGTAG, __VA_ARGS__)

#endif //MICROPHONE_LOGGING_H
