import Speech
import Foundation

guard CommandLine.arguments.count > 1 else {
    fputs("Usage: s2w_stt <wav_path> [lang]\n", stderr)
    exit(1)
}

let wavPath = CommandLine.arguments[1]
let lang = CommandLine.arguments.count > 2 ? CommandLine.arguments[2] : "zh-CN"

let locale = Locale(identifier: lang)
guard let recognizer = SFSpeechRecognizer(locale: locale) else {
    fputs("Cannot create recognizer for \(lang)\n", stderr)
    exit(1)
}

let url = URL(fileURLWithPath: wavPath)
let semaphore = DispatchSemaphore(value: 0)

var resultText = ""
var hadError = false

SFSpeechRecognizer.requestAuthorization { status in
    guard status == .authorized else {
        fputs("Speech recognition not authorized: \(status.rawValue)\n", stderr)
        hadError = true
        semaphore.signal()
        return
    }

    guard recognizer.isAvailable else {
        fputs("Recognizer not available\n", stderr)
        hadError = true
        semaphore.signal()
        return
    }

    let request = SFSpeechURLRecognitionRequest(url: url)
    request.requiresOnDeviceRecognition = recognizer.supportsOnDeviceRecognition
    request.taskHint = .dictation

    recognizer.recognitionTask(with: request) { result, error in
        if let error = error {
            fputs("Recognition error: \(error.localizedDescription)\n", stderr)
            hadError = true
            semaphore.signal()
            return
        }
        if let result = result, result.isFinal {
            resultText = result.bestTranscription.formattedString
            semaphore.signal()
        }
    }
}

_ = semaphore.wait(timeout: .now() + 60)

if hadError {
    exit(1)
}

if !resultText.isEmpty {
    print(resultText)
}

// Cleanup temp file
try? FileManager.default.removeItem(atPath: wavPath)
