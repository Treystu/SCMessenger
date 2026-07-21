//
//  QRCodeGenerator.swift
//  SCMessenger
//
//  Shared QR code bitmap generation, used by the identity export sheet
//  and safety-number verification sheet.
//

import CoreImage.CIFilterBuiltins
import UIKit

enum QRCodeGenerator {
    private static let context: CIContext = CIContext()

    /// Render `string` as a black-and-white QR code image, or nil if
    /// encoding fails (e.g. the payload is too large for a QR code's
    /// capacity).
    static func image(from string: String) -> UIImage? {
        let filter: CIFilter = CIFilter.qrCodeGenerator()
        let data: Data = Data(string.utf8)
        filter.setValue(data, forKey: "inputMessage")
        filter.setValue("Q", forKey: "inputCorrectionLevel")

        guard let outputImage = filter.outputImage else { return nil }
        let scaled: CIImage = outputImage.transformed(by: CGAffineTransform(scaleX: 12, y: 12))
        guard let cgImage = context.createCGImage(scaled, from: scaled.extent) else { return nil }
        return UIImage(cgImage: cgImage)
    }
}
