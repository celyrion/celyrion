cmake_minimum_required(VERSION 3.20)
project(celyrion_native_workers LANGUAGES CXX)

option(CELYRION_ENABLE_CUDA "Build CUDA kernels" OFF)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(celyrion_native STATIC src/stub.cpp)
target_include_directories(celyrion_native PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)

if(CELYRION_ENABLE_CUDA)
  enable_language(CUDA)
  target_sources(celyrion_native PRIVATE src/stub.cu)
  target_compile_definitions(celyrion_native PUBLIC CELYRION_HAS_CUDA=1)
endif()
