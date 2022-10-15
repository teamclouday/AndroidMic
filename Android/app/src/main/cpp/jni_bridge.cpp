#include "OboeRecorder.h"
#include "AudioBuffer.h"
#include "Logging.h"
#include <jni.h>
#include <chrono>
#include <thread>

extern "C"
JNIEXPORT void JNICALL
Java_com_example_androidmic_domain_audio_OboeRecorder_setDeviceId(JNIEnv *, jobject,
                                                           jint device_id) {
    auto recorder = OboeRecorder::Instance();
    recorder->SetDeviceId(device_id);
}

extern "C"
JNIEXPORT void JNICALL
Java_com_example_androidmic_domain_audio_OboeRecorder_setSampleRate(JNIEnv *, jobject,
                                                             jint sample_rate) {
    auto recorder = OboeRecorder::Instance();
    recorder->SetSampleRate(sample_rate);
}

extern "C"
JNIEXPORT void JNICALL
Java_com_example_androidmic_domain_audio_OboeRecorder_setBufferSizeInFrames(JNIEnv *, jobject,
                                                                     jint frames) {
    auto recorder = OboeRecorder::Instance();
    recorder->SetBufferSizeInFrames(frames);
}

extern "C"
JNIEXPORT void JNICALL
Java_com_example_androidmic_domain_audio_OboeRecorder_startRecordingInternal(JNIEnv *, jobject) {
    auto recorder = OboeRecorder::Instance();
    recorder->StartRecord();
}

extern "C"
JNIEXPORT void JNICALL
Java_com_example_androidmic_domain_audio_OboeRecorder_stopRecordingInternal(JNIEnv *, jobject) {
    auto recorder = OboeRecorder::Instance();
    recorder->StopRecord();
}

extern "C"
JNIEXPORT jint JNICALL
Java_com_example_androidmic_domain_audio_OboeRecorder_readInternalBytes(JNIEnv *env, jobject,
                                                                 jbyteArray buffer,
                                                                 jint offset, jint len) {
    auto recorder = OboeRecorder::Instance();
    auto readBuffer = recorder->GetAudioBuffer();
    if (!readBuffer || readBuffer->IsEmpty() || len <= 0) return 0;
    uint32_t size;
    const auto ptr = readBuffer->OpenReadMemoryRegion((uint32_t) len / 2, size);
    jint trueSize = (jint) size * 2;
    if (recorder->IsLittleEndian()) {
        env->SetByteArrayRegion(buffer, offset, trueSize,
                                reinterpret_cast<const jbyte *>(ptr));
    } else {
        // revert order of each short
        int8_t val;
        for (jint idx = 0; idx < (jint) size; ++idx) {
            auto shortVal = *(ptr + idx);
            val = (int8_t) ((shortVal >> 8) & 0xff);
            env->SetByteArrayRegion(buffer, offset + idx * 2, 1, &val);
            val = (int8_t) (shortVal & 0xff);
            env->SetByteArrayRegion(buffer, offset + idx * 2 + 1, 1, &val);
        }
    }
    readBuffer->CloseReadMemoryRegion(size);
    return trueSize;
}

extern "C"
JNIEXPORT jint JNICALL
Java_com_example_androidmic_domain_audio_OboeRecorder_readInternalShorts(JNIEnv *env, jobject,
                                                                  jshortArray buffer,
                                                                  jint offset, jint len) {
    auto recorder = OboeRecorder::Instance();
    auto readBuffer = recorder->GetAudioBuffer();
    if (!readBuffer || readBuffer->IsEmpty() || len <= 0) return 0;
    uint32_t size;
    const auto ptr = readBuffer->OpenReadMemoryRegion((uint32_t) len, size);
    env->SetShortArrayRegion(buffer, offset, (jsize) size, ptr);
    readBuffer->CloseReadMemoryRegion(size);
    return (jint) size;
}

extern "C"
JNIEXPORT jint JNICALL
Java_com_example_androidmic_domain_audio_OboeRecorder_readInternalBytesBlocking(JNIEnv *env, jobject,
                                                                         jbyteArray buffer,
                                                                         jint offset, jint len) {
    auto recorder = OboeRecorder::Instance();
    auto readBuffer = recorder->GetAudioBuffer();
    if (!readBuffer || readBuffer->IsEmpty() || len <= 0) return 0;
    jint toRead = len;
    uint32_t size;
    while (toRead > 0) {
        const auto ptr = readBuffer->OpenReadMemoryRegion((uint32_t) toRead / 2, size);
        jint trueSize = (jint) size * 2;
        if (recorder->IsLittleEndian()) {
            env->SetByteArrayRegion(buffer, offset, trueSize,
                                    reinterpret_cast<const jbyte *>(ptr));
        } else {
            // revert order of each short
            int8_t val;
            for (jint idx = 0; idx < (jint) size; ++idx) {
                auto shortVal = *(ptr + idx);
                val = (int8_t) ((shortVal >> 8) & 0xff);
                env->SetByteArrayRegion(buffer, offset + idx * 2, 1, &val);
                val = (int8_t) (shortVal & 0xff);
                env->SetByteArrayRegion(buffer, offset + idx * 2 + 1, 1, &val);
            }
        }
        toRead -= trueSize;
        offset += trueSize;
        readBuffer->CloseReadMemoryRegion(size);
    }
    return len;
}

extern "C"
JNIEXPORT jint JNICALL
Java_com_example_androidmic_domain_audio_OboeRecorder_readInternalShortsBlocking(JNIEnv *env, jobject,
                                                                          jshortArray buffer,
                                                                          jint offset, jint len) {
    auto recorder = OboeRecorder::Instance();
    auto readBuffer = recorder->GetAudioBuffer();
    if (!readBuffer || readBuffer->IsEmpty() || len <= 0) return 0;
    jint toRead = len;
    uint32_t size;
    while (toRead > 0) {
        const auto ptr = readBuffer->OpenReadMemoryRegion((uint32_t) toRead, size);
        env->SetShortArrayRegion(buffer, offset, (jsize) size, ptr);
        readBuffer->CloseReadMemoryRegion(size);
        toRead -= (jint) size;
        offset += (jint) size;
    }
    return len;
}