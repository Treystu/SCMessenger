//
//  Theme.swift
//  SCMessenger
//
//  Theme definitions matching Material Design equivalents
//

import SwiftUI

struct Theme {
    // MARK: - Colors (Material Design equivalents)

    /// Error container color (for relay toggle and critical warnings)
    static let errorContainer: Color = Color.red.opacity(0.12)
    static let onErrorContainer: Color = Color.red

    /// Primary container color
    static let primaryContainer: Color = Color.blue.opacity(0.12)
    static let onPrimaryContainer: Color = Color.blue

    /// Success color
    static let successContainer: Color = Color.green.opacity(0.12)
    static let onSuccessContainer: Color = Color.green

    /// Warning color
    static let warningContainer: Color = Color.orange.opacity(0.12)
    static let onWarningContainer: Color = Color.orange

    /// Surface colors
    static let surface: Color = Color(uiColor: .systemBackground)
    static let surfaceVariant: Color = Color(uiColor: .secondarySystemBackground)
    static let onSurface: Color = Color(uiColor: .label)
    static let onSurfaceVariant: Color = Color(uiColor: .secondaryLabel)

    // MARK: - Typography

    static let displayLarge: Font = Font.system(size: 57, weight: .regular)
    static let displayMedium: Font = Font.system(size: 45, weight: .regular)
    static let displaySmall: Font = Font.system(size: 36, weight: .regular)

    static let headlineLarge: Font = Font.system(size: 32, weight: .regular)
    static let headlineMedium: Font = Font.system(size: 28, weight: .regular)
    static let headlineSmall: Font = Font.system(size: 24, weight: .regular)

    static let titleLarge: Font = Font.system(size: 22, weight: .regular)
    static let titleMedium: Font = Font.system(size: 16, weight: .medium)
    static let titleSmall: Font = Font.system(size: 14, weight: .medium)

    static let bodyLarge: Font = Font.system(size: 16, weight: .regular)
    static let bodyMedium: Font = Font.system(size: 14, weight: .regular)
    static let bodySmall: Font = Font.system(size: 12, weight: .regular)

    static let labelLarge: Font = Font.system(size: 14, weight: .medium)
    static let labelMedium: Font = Font.system(size: 12, weight: .medium)
    static let labelSmall: Font = Font.system(size: 11, weight: .medium)

    // MARK: - Spacing

    static let spacingXSmall: CGFloat = 4
    static let spacingSmall: CGFloat = 8
    static let spacingMedium: CGFloat = 16
    static let spacingLarge: CGFloat = 24
    static let spacingXLarge: CGFloat = 32

    // MARK: - Corner Radius

    static let cornerRadiusSmall: CGFloat = 8
    static let cornerRadiusMedium: CGFloat = 12
    static let cornerRadiusLarge: CGFloat = 16
    static let cornerRadiusXLarge: CGFloat = 28

    // MARK: - Elevation (Shadow)

    static let elevation1: Shadow = Shadow(radius: 1, y: 1)
    static let elevation2: Shadow = Shadow(radius: 3, y: 2)
    static let elevation3: Shadow = Shadow(radius: 6, y: 4)

    struct Shadow {
        let radius: CGFloat
        let y: CGFloat
    }
}

// MARK: - View Extensions

extension View {
    func themedCard(elevation: Theme.Shadow = Theme.elevation1) -> some View {
        self
            .background(Theme.surface)
            .cornerRadius(Theme.cornerRadiusMedium)
            .shadow(radius: elevation.radius, y: elevation.y)
    }

    func errorContainerStyle() -> some View {
        self
            .padding(Theme.spacingMedium)
            .background(Theme.errorContainer)
            .cornerRadius(Theme.cornerRadiusSmall)
    }

    func primaryContainerStyle() -> some View {
        self
            .padding(Theme.spacingMedium)
            .background(Theme.primaryContainer)
            .cornerRadius(Theme.cornerRadiusSmall)
    }
}