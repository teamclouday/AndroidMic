#include "OboeRecorder.h"
#include <jni.h>

extern "C"
JNIEXPORT void JNICALL
Java_com_example_microphone_audio_OboeRecorder_setDeviceId(JNIEnv *, jobject,
                                                           jint device_id) {
    auto recorder = OboeRecorder::Instance();
    recorder->SetDeviceId(device_id);
}

extern "C"
JNIEXPORT void JNICALL
Java_com_example_microphone_audio_OboeRecorder_setSampleRate(JNIEnv *, jobject,
                                                             jint sample_rate) {
    auto recorder = OboeRecorder::Instance();
    recorder->SetSampleRate(sample_rate);
}

extern "C"
JNIEXPORT void JNICALL
Java_com_example_microphone_audio_OboeRecorder_setBufferSizeInFrames(JNIEnv *, jobject,
                                                                     jint frames) {
    auto recorder = OboeRecorder::Instance();
    recorder->SetBufferSizeInFrames(frames);
}

extern "C"
JNIEXPORT void JNICALL
Java_com_example_microphone_audio_OboeRecorder_startRecordingInternal(JNIEnv *, jobject) {
    auto recorder = OboeRecorder::Instance();
    recorder->StartRecord();
}

extern "C"
JNIEXPORT void JNICALL
Java_com_example_microphone_audio_OboeRecorder_stopRecordingInternal(JNIEnv *, jobject) {
    auto recorder = OboeRecorder::Instance();
    recorder->StopRecord();
}

extern "C"
JNIEXPORT jbyteArray JNICALL
Java_com_example_microphone_audio_OboeRecorder_readInternalBytes(JNIEnv *env, jobject) {
    auto recorder = OboeRecorder::Instance();
    if (!recorder->HasBuffer()) return nullptr;
    auto &lock = recorder->GetLock();
    lock.lock();
    auto recorded = recorder->GetRecordedFrames() * 2;
    jbyteArray bytes = nullptr;
    if (recorded) {
        bytes = env->NewByteArray(recorded);
        auto region = recorder->GetByteBuffer();
        env->SetByteArrayRegion(bytes, 0, recorded, region);
    }
    recorder->ReleaseBuffer();
    lock.unlock();
    return bytes;
}

extern "C"
JNIEXPORT jint JNICALL
Java_com_example_microphone_audio_OboeRecorder_readInternalShorts(JNIEnv *env, jobject,
                                                                  jshortArray buffer,
                                                                  jint num_shorts) {
    auto recorder = OboeRecorder::Instance();
    if (!recorder->HasBuffer()) return 0;
    auto &lock = recorder->GetLock();
    lock.lock();
    auto region = recorder->GetBuffer();
    auto recorded = recorder->GetRecordedFrames();
    if (num_shorts < recorded) recorded = num_shorts;
    env->SetShortArrayRegion(buffer, 0, recorded, region);
    recorder->ReleaseBuffer();
    lock.unlock();
    return recorded;
}
