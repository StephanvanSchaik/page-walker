# Introduction

[![Crates.io](https://img.shields.io/crates/v/page-walker.svg)](https://crates.io/crates/page-walker)
[![Docs](https://docs.rs/page-walker/badge.svg)](https://docs.rs/page-walker)

This crate implements a generic page table walker in Rust, which can be used to either introspect
or manage virtual address spaces on architectures that implement a Memory Management Unit (MMU)
that traverses a hierarchy of page tables to translate virtual address into physical addresses and
a set of permissions. Note that paging is not limited to CPUs, and that paging is also common on
modern GPUs. The implementations provided here may therefore be useful when implementing drivers
for any sort of paging architecture, an operating system, a hypervisor, etc.
