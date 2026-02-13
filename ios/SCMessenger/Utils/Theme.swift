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
    static let errorContainer = Color.red.opacity(0.12)
    static let onErrorContainer = Color.red
    
    /// Primary container color
    static let primaryContainer = Color.blue.opacity(0.12)
    static let onPrimaryContainer = Color.blue
    
    /// Success color
    static let successContainer = Color.green.opacity(0.12)
    static let onSuccessContainer = Color.green
    
    /// Warning color
    static let warningContainer = Color.orange.opacity(0.12)
    static let onWarningContainer = Color.orange
    
    /// Surface colors
    static let surface = Color(uiColor: .systemBackground)
    static let surfaceVariant = Color(uiColor: .secondarySystemBackground)
    static let onSurface = Color(uiColor: .label)
    static let onSurfaceVariant = Color(uiColor: .secondaryLabel)
    
    // MARK: - Typography
    
    static let displayLarge = Font.system(size: 57, weight: .regular)
    static let displayMedium = Font.system(size: 45, weight: .regular)
    static let displaySmall = Font.system(size: 36, weight: .regular)
    
    static let headlineLarge = Font.system(size: 32, weight: .regular)
    static let headlineMedium = Font.system(size: 28, weight: .regular)
    static let headlineSmall = Font.system(size: 24, weight: .regular)
    
    static let titleLarge = Font.system(size: 22, weight: .regular)
    static let titleMedium = Font.system(size: 16, weight: .medium)
    static let titleSmall = Font.system(size: 14, weight: .medium)
    
    static let bodyLarge = Font.system(size: 16, weight: .regular)
    static let bodyMedium = Font.system(size: 14, weight: .regular)
    static let bodySmall = Font.system(size: 12, weight: .regular)
    
    static let labelLarge = Font.system(size: 14, weight: .medium)
    static let labelMedium = Font.system(size: 12, weight: .medium)
    static let labelSmall = Font.system(size: 11, weight: .medium)
    
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
    
    static let elevation1 = Shadow(radius: 1, y: 1)
    static let elevation2 = Shadow(radius: 3, y: 2)
    static let elevation3 = Shadow(radius: 6, y: 4)
    
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
