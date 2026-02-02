package io.github.teamclouday.androidMic.ui.home.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import io.github.teamclouday.androidMic.Mode
import io.github.teamclouday.androidMic.ui.MainViewModel

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