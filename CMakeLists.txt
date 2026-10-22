cmake_minimum_required(VERSION 3.12)

project(librespot VERSION 1.0.0 LANGUAGES C)

include(GNUInstallDirs)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)
set(CMAKE_STATIC_LIBRARY_PREFIX "")

file(GLOB HEADER_FILES CONFIGURE_DEPENDS
    include/${PROJECT_NAME}/*.h
)

set(EMPTY_SRC "${CMAKE_BINARY_DIR}/_.c")
file(WRITE ${EMPTY_SRC} "// auto-generated empty source\nvoid __empty(void) {}")

add_library(${PROJECT_NAME} STATIC ${EMPTY_SRC} ${HEADER_FILES})
set_target_properties(${PROJECT_NAME} PROPERTIES LINKER_LANGUAGE C)
target_include_directories(${PROJECT_NAME}
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)

if(WIN32)
    set(RUST_LIB_NAME "librespot_core.dll")
    set(RUST_LIB_INSTALL_DIR "${CMAKE_INSTALL_BINDIR}")
    if(MSVC)
        set(RUST_IMPORT_LIB_NAME "librespot_core.dll.lib")
    else()
        set(RUST_IMPORT_LIB_NAME "librespot_core.dll.a")
    endif()
elseif(APPLE)
    set(RUST_LIB_NAME "librespot_core.dylib")
    set(RUST_LIB_INSTALL_DIR "${CMAKE_INSTALL_LIBDIR}")
    set(RUST_IMPORT_LIB_NAME "")
else()
    set(RUST_LIB_NAME "librespot_core.so")
    set(RUST_LIB_INSTALL_DIR "${CMAKE_INSTALL_LIBDIR}")
    set(RUST_IMPORT_LIB_NAME "")
endif()
set(RUST_LIB_PATH "${CMAKE_CURRENT_SOURCE_DIR}/core/target/release/${RUST_LIB_NAME}")

add_custom_target(
    ${PROJECT_NAME}_rust
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/core
)
add_custom_command(
    TARGET ${PROJECT_NAME}_rust
    POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_BINARY_DIR}/lib
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
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
)

install(
    FILES ${HEADER_FILES}
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/${PROJECT_NAME}
)

install(
    EXPORT ${PROJECT_NAME}Targets
    FILE ${PROJECT_NAME}Targets.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}
)

configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/config.cmake.in
    ${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake
    @ONLY
)

install(
    FILES ${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}
)

# install rust core lib
install(
    FILES ${CMAKE_BINARY_DIR}/lib/${RUST_LIB_NAME}
    DESTINATION ${RUST_LIB_INSTALL_DIR}
)
