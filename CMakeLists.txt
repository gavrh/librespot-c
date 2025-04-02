cmake_minimum_required(VERSION 3.12)

project(librespot VERSION 1.0.0 LANGUAGES C)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)
set(CMAKE_STATIC_LIBRARY_PREFIX "")

file(GLOB_RECURSE HEADER_FILES CONFIGURE_DEPENDS
    include/${PROJECT_NAME}/*.h
)

add_library(${PROJECT_NAME} STATIC ${HEADER_FILES})
set_target_properties(${PROJECT_NAME} PROPERTIES LINKER_LANGUAGE C)
target_include_directories(${PROJECT_NAME}
    INTERFACE
        $<INSTALL_INTERFACE:include>
        $<BUILD_INTERFACE:${CMAKE_SOURCE_DIR}/include>
    PUBLIC
        $<INSTALL_INTERFACE:include>
    PRIVATE
        $<BUILD_INTERFACE:${CMAKE_SOURCE_DIR}/include>
)

if(WIN32)
    set(RUST_LIB_NAME "librespot_core.dll")
elseif(APPLE)
    set(RUST_LIB_NAME "librespot_core.dylib")
else()
    set(RUST_LIB_NAME "librespot_core.so")
endif()
set(RUST_LIB_PATH "${CMAKE_SOURCE_DIR}/core/target/release/${RUST_LIB_NAME}")

add_custom_target(
    ${PROJECT_NAME}_rust
    COMMAND cargo build --release
    WORKING_DIRECTORY ${${PROJECT_NAME}_SOURCE_DIR}/core
)
add_custom_command(
    TARGET ${PROJECT_NAME}_rust
    POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_LIB_PATH}
        ${CMAKE_BINARY_DIR}/lib/${RUST_LIB_NAME}
)

add_dependencies(${PROJECT_NAME} ${PROJECT_NAME}_rust)
target_link_libraries(${PROJECT_NAME} PRIVATE ${CMAKE_BINARY_DIR}/lib/${RUST_LIB_NAME})

# install librespot
install(
    TARGETS ${PROJECT_NAME}
    EXPORT ${PROJECT_NAME}Targets
    LIBRARY DESTINATION lib
    PUBLIC_HEADER DESTINATION include
)

install(
    DIRECTORY ${CMAKE_SOURCE_DIR}/include/${PROJECT_NAME}/
    DESTINATION include/${PROJECT_NAME}
)

install(
    EXPORT ${PROJECT_NAME}Targets
    FILE ${PROJECT_NAME}Targets.cmake
    DESTINATION lib/cmake/${PROJECT_NAME}
)

configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/config.cmake.in
    ${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake
    @ONLY
)

install(
    FILES ${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake
    DESTINATION lib/cmake/${PROJECT_NAME}
)

# install rust core lib
install(
    FILES ${CMAKE_BINARY_DIR}/lib/${RUST_LIB_NAME}
    DESTINATION lib
)
