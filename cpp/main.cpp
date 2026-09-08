#include "zenoh.hxx"
#include <chrono>
#include <cstdlib>
#include <iostream>
#include <thread>

int main() {
    try {
        auto config = zenoh::Config::from_file(ZENOH_CONFIG_PATH);
        auto session = zenoh::Session::open(std::move(config));
        auto subscriber = session.declare_subscriber(
            zenoh::KeyExpr("demo/example/**"),
            [](const zenoh::Sample& sample) {
                std::cout << "RECV " << sample.get_keyexpr().as_string_view() << " "
                          << sample.get_payload().as_string() << std::endl;
            }, zenoh::closures::none);
        auto publisher = session.declare_publisher(zenoh::KeyExpr("demo/example/cpp"));
        std::cout << "READY cpp" << std::endl;
        const auto* duration = std::getenv("DEMO_SECONDS");
        const int seconds = duration ? std::stoi(duration) : 30;
        for (int i = 0; i < seconds; ++i) {
            std::this_thread::sleep_for(std::chrono::seconds(1));
            publisher.put("Hello from C++! " + std::to_string(i));
        }
    } catch (const std::exception& error) {
        std::cerr << error.what() << std::endl;
        return 1;
    }
}
