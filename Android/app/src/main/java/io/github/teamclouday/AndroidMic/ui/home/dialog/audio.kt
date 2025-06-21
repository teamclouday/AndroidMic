package io.github.teamclouday.AndroidMic.ui.home.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import io.github.teamclouday.AndroidMic.AudioFormat
import io.github.teamclouday.AndroidMic.ChannelCount
import io.github.teamclouday.AndroidMic.SampleRates
import io.github.teamclouday.AndroidMic.ui.MainViewModel

@Composable
fun DialogSampleRate(
    vm: MainViewModel,
    expanded: MutableState<Boolean>,
) {
    DialogList(
        expanded,
        enum = SampleRates.entries,
        onClick = { vm.setSampleRate(it) },
        text = { it.value.toString() }
    )
}

@Composable
fun DialogChannelCount(
    vm: MainViewModel,
    expanded: MutableState<Boolean>,
) {
    DialogList(
        expanded,
        enum = ChannelCount.entries,
        onClick = { vm.setChannelCount(it) },
        text = { it.toString() }
    )
}

@Composable
fun DialogAudioFormat(
    vm: MainViewModel,
    expanded: MutableState<Boolean>,
) {
    DialogList(
        expanded,
        enum = AudioFormat.entries,
        onClick = { vm.setAudioFormat(it) },
        text = { it.toString() }
    )
}