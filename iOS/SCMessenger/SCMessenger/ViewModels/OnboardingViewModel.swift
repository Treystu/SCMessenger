//
//  OnboardingViewModel.swift
//  SCMessenger
//
//  ViewModel for onboarding flow
//

import Foundation

@Observable
final class OnboardingViewModel {
    private let onboardingKey = "hasCompletedOnboarding"
    private let installChoiceKey = "hasCompletedInstallModeChoice"

    var currentStep = 0
    var hasCompletedOnboarding = false
    
    let totalSteps = 4
    
    func advance() {
        if currentStep < totalSteps - 1 {
            currentStep += 1
        }
    }
    
    func goBack() {
        if currentStep > 0 {
            currentStep -= 1
        }
    }
    
    func completeOnboarding() {
        hasCompletedOnboarding = true
        UserDefaults.standard.set(true, forKey: onboardingKey)
        UserDefaults.standard.set(true, forKey: installChoiceKey)
    }
    
    func skip() {
        completeOnboarding()
    }
}
