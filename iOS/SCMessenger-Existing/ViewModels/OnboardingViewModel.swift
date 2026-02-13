//
//  OnboardingViewModel.swift
//  SCMessenger
//
//  ViewModel for onboarding flow
//

import Foundation

@Observable
final class OnboardingViewModel {
    var currentStep = 0
    var hasCompletedOnboarding = false
    
    let totalSteps = 5
    
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
        UserDefaults.standard.set(true, forKey: "hasCompletedOnboarding")
    }
    
    func skip() {
        completeOnboarding()
    }
}
