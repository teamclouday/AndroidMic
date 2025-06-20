package io.github.teamclouday.androidMic.ui.home.dialog

import android.os.Build
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.res.stringResource
import io.github.teamclouday.androidMic.R
import io.github.teamclouday.androidMic.Themes
import io.github.teamclouday.androidMic.ui.MainViewModel
import io.github.teamclouday.androidMic.ui.components.ManagerCheckBox

@Composable
fun DialogTheme(
    vm: MainViewModel,
    expanded: MutableState<Boolean>,
) {
    DialogList(
        expanded,
        enum = Themes.entries,
        onClick = { vm.setTheme(it) },
        text = { it.toString() }
    ) {
        if (Build.VERSION.SDK_INT > Build.VERSION_CODES.S) {
            DialogDivider()

            ManagerCheckBox(
                checked = vm.prefs.dynamicColor.getAsState().value,
                onClick = {
                    vm.setDynamicColor(it)
                },
                text = stringResource(id = R.string.dynamic_color)
            )
        }
    }
}
