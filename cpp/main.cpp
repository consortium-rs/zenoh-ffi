#include "zenoh.hxx"
#include "demo.pb.h"
#include <opencv2/core.hpp>
#include <opencv2/imgproc.hpp>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <print>
#include <ranges>
#include <stdexcept>
#include <string>
#include <thread>

using namespace std::chrono_literals;

int main() {
    try {
        const auto* duration = std::getenv("DEMO_SECONDS");
        const int seconds = duration ? std::stoi(duration) : 30;
        if (seconds < 1) {
            throw std::invalid_argument("DEMO_SECONDS must be positive");
        }
        auto config = zenoh::Config::from_file(ZENOH_CONFIG_PATH);
        auto session = zenoh::Session::open(std::move(config));
        auto subscriber = session.declare_subscriber(
            zenoh::KeyExpr("demo/example/**"),
            [](const zenoh::Sample& sample) {
                demo::Result result;
                if (!result.ParseFromString(sample.get_payload().as_string())) {
                    std::println(stderr, "Invalid protobuf payload");
                    return;
                }
                std::println("RECV {} {} {}", sample.get_keyexpr().as_string_view(),
                             result.language(), result.operation());
                std::fflush(stdout);
            }, zenoh::closures::none);
        auto publisher = session.declare_publisher(zenoh::KeyExpr("demo/example/cpp"));
        // Threshold a tiny grayscale image: two pixels exceed 127.
        const cv::Mat image = (cv::Mat_<unsigned char>(2, 2) << 0, 100, 200, 255);
        cv::Mat mask;
        cv::threshold(image, mask, 127, 255, cv::THRESH_BINARY);
        const auto count = cv::countNonZero(mask);
        std::println("READY cpp");
        std::fflush(stdout);
        if (const auto* gate = std::getenv("DEMO_START_FILE")) {
            const auto deadline = std::chrono::steady_clock::now() + 60s;
            while (!std::filesystem::exists(gate)) {
                if (std::chrono::steady_clock::now() > deadline) {
                    throw std::runtime_error("Startup barrier timed out");
                }
                std::this_thread::sleep_for(50ms);
            }
        }
        for (const int i : std::views::iota(0, seconds)) {
            std::this_thread::sleep_for(1s);
            demo::Result result;
            result.set_language("cpp");
            result.set_sequence(i);
            result.set_operation("opencv.threshold");
            result.add_values(count);
            publisher.put(result.SerializeAsString());
        }
    } catch (const std::exception& error) {
        std::println(stderr, "{}", error.what());
        return 1;
    }
}
