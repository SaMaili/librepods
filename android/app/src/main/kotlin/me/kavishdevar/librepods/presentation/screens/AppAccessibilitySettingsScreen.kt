package me.kavishdevar.librepods.presentation.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import me.kavishdevar.librepods.R
import me.kavishdevar.librepods.presentation.components.primitives.StyledList
import me.kavishdevar.librepods.presentation.components.primitives.StyledScaffold
import me.kavishdevar.librepods.presentation.components.primitives.styledToggle
import me.kavishdevar.librepods.presentation.viewmodel.AppSettingsViewModel

@Composable
fun AppAccessibilitySettingsRoute(
    viewModel: AppSettingsViewModel,
    navigateBack: (() -> Unit)?
) {
    val uiState by viewModel.uiState.collectAsState()
    val settings = uiState.settings
    val accessibilitySettings = settings.accessibilitySettings
    AppAccessibilitySettingsScreen(
        navigateBack = navigateBack,
        differentiateWithoutColor = accessibilitySettings.differentiateWithoutColor,
        onDifferentiateWithoutColorChanged = { differentiateWithoutColor ->
            viewModel.updateSettings {
                it.copy(accessibilitySettings = accessibilitySettings.copy(differentiateWithoutColor = differentiateWithoutColor))
            }
        },
    )
}

@Composable
fun AppAccessibilitySettingsScreen(
    navigateBack: (() -> Unit)? = null,
    differentiateWithoutColor: Boolean,
    onDifferentiateWithoutColorChanged: (Boolean) -> Unit
) {
    val scrollState = rememberScrollState()

    StyledScaffold(
        title = stringResource(R.string.accessibility),
        navigateBack = navigateBack
    ) { topPadding, bottomPadding ->
        Column(
            modifier = Modifier
                .padding(horizontal = 16.dp)
                .verticalScroll(scrollState),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Spacer(modifier = Modifier.padding(top = topPadding))

            StyledList {
                styledToggle(
                    label = stringResource(R.string.differentiate_without_color),
                    checked = differentiateWithoutColor,
                    onCheckedChange = onDifferentiateWithoutColorChanged
                )
            }

            Spacer(modifier = Modifier.padding(bottom = bottomPadding))
        }
    }
}
