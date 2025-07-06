package io.github.teamclouday.AndroidMic.ui.home.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import io.github.teamclouday.AndroidMic.Mode
import io.github.teamclouday.AndroidMic.ui.MainViewModel

@Composable
fun DialogMode(
    vm: MainViewModel,
    expanded: MutableState<Boolean>,
) {
    DialogList(
        expanded,
        enum = Mode.entries,
        onClick = { vm.setMode(it) },
        text = { it.toString() }
    )
}