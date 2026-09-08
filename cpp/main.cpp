#include "zenoh.hxx"
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <format>
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
        if (seconds < 0) {
            throw std::invalid_argument("DEMO_SECONDS must be nonnegative");
        }
        auto config = zenoh::Config::from_file(ZENOH_CONFIG_PATH);
        auto session = zenoh::Session::open(std::move(config));
        auto subscriber = session.declare_subscriber(
            zenoh::KeyExpr("demo/example/**"),
            [](const zenoh::Sample& sample) {
                std::println("RECV {} {}", sample.get_keyexpr().as_string_view(),
                             sample.get_payload().as_string());
                std::fflush(stdout);
            }, zenoh::closures::none);
        auto publisher = session.declare_publisher(zenoh::KeyExpr("demo/example/cpp"));
        std::println("READY cpp");
        std::fflush(stdout);
        for (const int i : std::views::iota(0, seconds)) {
            std::this_thread::sleep_for(1s);
            publisher.put(std::format("Hello from C++! {}", i));
        }
    } catch (const std::exception& error) {
        std::println(stderr, "{}", error.what());
        return 1;
    }
}
