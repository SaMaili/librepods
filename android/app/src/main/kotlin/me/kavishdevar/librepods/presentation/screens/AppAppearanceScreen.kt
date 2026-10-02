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
import me.kavishdevar.librepods.data.app.FontFamilyOption
import me.kavishdevar.librepods.data.app.FontSettings
import me.kavishdevar.librepods.presentation.components.primitives.StyledList
import me.kavishdevar.librepods.presentation.components.primitives.StyledScaffold
import me.kavishdevar.librepods.presentation.components.primitives.styledListItem
import me.kavishdevar.librepods.presentation.design.DesignSystem
import me.kavishdevar.librepods.presentation.design.NightTheme
import me.kavishdevar.librepods.presentation.viewmodel.AppSettingsViewModel

@Composable
fun AppAppearanceSettingsRoute(
    viewModel: AppSettingsViewModel,
    navigateBack: (() -> Unit)?
) {
    val uiState by viewModel.uiState.collectAsState()
    val settings = uiState.settings

    AppAppearanceSettingsScreen(
        navigateBack = navigateBack,
        designSystem = settings.designSystem,
        onDesignSystemChanged = { designSystem ->
            viewModel.updateSettings {
                it.copy(designSystem = designSystem)
            }
        },
        nightMode = settings.nightMode,
        onNightModeChanged = { nightMode ->
            viewModel.updateSettings {
                it.copy(nightMode = nightMode)
            }
        },
        fontSettings = settings.fontSettings,
        onFontSettingsChanged = { fontSettings ->
            viewModel.updateSettings {
                it.copy(fontSettings = fontSettings)
            }
        }
    )
}

@Composable
fun AppAppearanceSettingsScreen(
    navigateBack: (() -> Unit)? = null,
    designSystem: DesignSystem,
    onDesignSystemChanged: (DesignSystem) -> Unit,
    nightMode: NightTheme,
    onNightModeChanged: (NightTheme) -> Unit,
    fontSettings: FontSettings,
    onFontSettingsChanged: (FontSettings) -> Unit,
) {
    val scrollState = rememberScrollState()

    StyledScaffold(
        title = stringResource(R.string.appearance),
        navigateBack = navigateBack
    ) { topPadding, bottomPadding ->
        Column(
            modifier = Modifier
                .padding(horizontal = 16.dp)
                .verticalScroll(scrollState),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Spacer(modifier = Modifier.padding(top = topPadding))

            StyledList(title = stringResource(R.string.appearance)) {
                styledListItem(
                    contentText = stringResource(R.string.light),
                    selected = nightMode == NightTheme.Light,
                    onClick = { onNightModeChanged(NightTheme.Light) },
                )

                styledListItem(
                    contentText = stringResource(R.string.system),
                    selected = nightMode == NightTheme.System,
                    onClick = { onNightModeChanged(NightTheme.System) },
                )

                styledListItem(
                    contentText = stringResource(R.string.dark),
                    selected = nightMode == NightTheme.Dark,
                    onClick = { onNightModeChanged(NightTheme.Dark) },
                )
            }

            StyledList(
                title = stringResource(R.string.design_system),
            ) {
                styledListItem(
                    contentText = stringResource(R.string.apple),
                    selected = designSystem == DesignSystem.Apple,
                    onClick = { onDesignSystemChanged(DesignSystem.Apple) },
                )

                styledListItem(
                    contentText = stringResource(R.string.material3e),
                    selected = designSystem == DesignSystem.Material,
                    onClick = { onDesignSystemChanged(DesignSystem.Material) },
                )
            }

            StyledList(title = stringResource(R.string.font)) {
                styledListItem(
                    contentText = stringResource(R.string.system),
                    selected = fontSettings.fontFamilyOption == FontFamilyOption.SystemDefault,
                    onClick = { onFontSettingsChanged(fontSettings.copy(fontFamilyOption = FontFamilyOption.SystemDefault)) },
                )
                styledListItem(
                    contentText = stringResource(R.string.roboto_flex),
                    selected = fontSettings.fontFamilyOption == FontFamilyOption.RobotoFlex,
                    onClick = { onFontSettingsChanged(fontSettings.copy(fontFamilyOption = FontFamilyOption.RobotoFlex)) },
                )
                styledListItem(
                    contentText = stringResource(R.string.roboto_flex_expressive),
                    selected = fontSettings.fontFamilyOption == FontFamilyOption.RobotoFlexExpressive,
                    onClick = { onFontSettingsChanged(fontSettings.copy(fontFamilyOption = FontFamilyOption.RobotoFlexExpressive)) },
                )
                styledListItem(
                    contentText = stringResource(R.string.inter),
                    selected = fontSettings.fontFamilyOption == FontFamilyOption.Inter,
                    onClick = { onFontSettingsChanged(fontSettings.copy(fontFamilyOption = FontFamilyOption.Inter)) },
                )
            }

            Spacer(modifier = Modifier.padding(bottom = bottomPadding))
        }
    }
}
