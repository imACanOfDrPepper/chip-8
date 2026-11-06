use super::*;

fn run_test(opcode: u16, setup: impl FnOnce(&mut Cpu)) -> Cpu {
    let mut cpu = Cpu::new();

    setup(&mut cpu);

    cpu.memory[cpu.pc as usize] = (opcode >> 8) as u8;
    cpu.memory[(cpu.pc + 1) as usize] = (opcode & 0x00FF) as u8;

    cpu.cycle([false; 16]);
    cpu
}

#[test]
fn test_advance_pc() {
    assert_eq!(
        run_test(0x00E0, |_| {}).pc,
        0x202,
        "PC should advance after an instruction"
    );
}

#[test]
fn test_pc_overflow() {
    assert_eq!(
        run_test(0x00E0, |c| { c.pc = 0xFFE }).pc,
        0x000,
        "PC should overflow"
    );
}

#[test]
fn test_cls() {
    assert_eq!(
        run_test(0x00E0, |c| { c.frame_buffer[0] = 1 }).frame_buffer[0],
        0x0,
        "CLS should clear the screen"
    );
}

#[test]
fn test_ret() {
    let cpu = run_test(0x00EE, |c| {
        c.stack[0] = 0x3939;
        c.stack_pointer = 0x1
    });
    assert_eq!(
        cpu.stack_pointer, 0x0,
        "RET should decrement the stack pointer"
    );
    assert_eq!(cpu.pc, 0x0939, "RET should overwrite the PC");
}

#[test]
fn test_ret_underflow() {
    let cpu = run_test(0x00EE, |_| {});
    assert_eq!(cpu.stack_pointer, 0xF, "Stack pointer should underflow");
    assert_eq!(cpu.pc, 0x000, "RET should overwrite the PC");
}

#[test]
fn test_jmp() {
    assert_eq!(
        run_test(0x1420, |_| {}).pc,
        0x420,
        "JMP should affect the PC"
    );
}

#[test]
fn test_cal() {
    let cpu = run_test(0x2420, |_| {});
    assert_eq!(
        cpu.stack[0x0], 0x202,
        "CAL should write to the stack at the correct place"
    );
    assert_eq!(
        cpu.stack_pointer, 0x1,
        "CAL should increment the stack pointer"
    );
    assert_eq!(cpu.pc, 0x420, "CAL should overwrite the PC");
}

#[test]
fn test_cal_overflow() {
    let cpu = run_test(0x2420, |c| c.stack_pointer = 0xF);
    assert_eq!(
        cpu.stack[0xF], 0x202,
        "CAL should write to the stack at the correct place"
    );
    assert_eq!(cpu.stack_pointer, 0x0, "Stack pointer should underflow");
    assert_eq!(cpu.pc, 0x420, "CAL should overwrite the PC");
}

#[test]
fn test_seb_eq() {
    assert_eq!(
        run_test(0x3969, |c| { c.v[0x9] = 0x69 }).pc,
        0x204,
        "SEB should skip next instruction if Vx and kk are equal"
    );
}

#[test]
fn test_seb_ne() {
    assert_eq!(
        run_test(0x3969, |c| { c.v[0x9] = 0x39 }).pc,
        0x202,
        "SEB should do nothing if Vx and kk are not equal"
    );
}

#[test]
fn test_sne_eq() {
    assert_eq!(
        run_test(0x4269, |c| { c.v[0x2] = 0x69 }).pc,
        0x202,
        "SNE should do nothing if Vx and kk are equal"
    );
}

#[test]
fn test_sne_ne() {
    assert_eq!(
        run_test(0x4269, |c| { c.v[0x2] = 0x39 }).pc,
        0x204,
        "SNE should skip next instruction if Vx and kk are not equal"
    );
}

#[test]
fn test_sev_eq() {
    assert_eq!(
        run_test(0x5420, |c| {
            c.v[0x4] = 0x39;
            c.v[0x2] = 0x39
        })
        .pc,
        0x204,
        "SEV should skip next instruction if Vx and Vy are equal"
    );
}

#[test]
fn test_sev_ne() {
    assert_eq!(
        run_test(0x5420, |c| {
            c.v[0x4] = 0x39;
            c.v[0x2] = 0x42
        })
        .pc,
        0x202,
        "SEV should do nothing if Vx and Vy are not equal"
    );
}

#[test]
fn test_ldvb() {
    assert_eq!(
        run_test(0x6969, |_| {}).v[0x9],
        0x69,
        "LDVB should change Vx correctly"
    );
}

#[test]
fn test_add() {
    assert_eq!(
        run_test(0x7777, |c| { c.v[0x7] = 0x01 }).v[0x7],
        0x78,
        "ADD should add correctly"
    );
}

#[test]
fn test_ldvv() {
    assert_eq!(
        run_test(0x8120, |c| {
            c.v[1] = 0x69;
            c.v[2] = 0x39
        })
        .v[0x1],
        0x39,
        "LDVV should change Vx correctly"
    );
}

#[test]
fn test_or() {
    assert_eq!(
        run_test(0x8121, |c| {
            c.v[1] = 0x69;
            c.v[2] = 0x39
        })
        .v[0x1],
        0x79,
        "OR should perform the correct operation"
    );
}

#[test]
fn test_and() {
    assert_eq!(
        run_test(0x8122, |c| {
            c.v[1] = 0x69;
            c.v[2] = 0x39
        })
        .v[0x1],
        0x29,
        "AND should perform the correct operation"
    );
}

#[test]
fn test_xor() {
    assert_eq!(
        run_test(0x8123, |c| {
            c.v[1] = 0x69;
            c.v[2] = 0x39
        })
        .v[0x1],
        0x50,
        "XOR should perform the correct operation"
    );
}

#[test]
fn test_addc_coff() {
    let cpu = run_test(0x8124, |c| {
        c.v[1] = 0x69;
        c.v[2] = 0x39
    });
    assert_eq!(cpu.v[0x1], 0xA2, "ADDC should add correctly");
    assert_eq!(cpu.v[0xF], 0x00, "ADDC should set VF to 0 if Vx+Vy < 0x100");
}

#[test]
fn test_addc_con() {
    let cpu = run_test(0x8124, |c| {
        c.v[1] = 0x69;
        c.v[2] = 0xFF
    });
    assert_eq!(cpu.v[0x1], 0x68, "ADDC should overflow the result");
    assert_eq!(
        cpu.v[0xF], 0x01,
        "ADDC should set VF to 1 if Vx+Vy >= 0x100"
    );
}

#[test]
fn test_subc_coff() {
    let cpu = run_test(0x8125, |c| {
        c.v[1] = 0x39;
        c.v[2] = 0x69
    });
    assert_eq!(cpu.v[0x1], 0xD0, "SUBC should underflow the result");
    assert_eq!(cpu.v[0xF], 0x00, "SUBC should set VF to 0 if Vx-Vy < 0");
}

#[test]
fn test_subc_con() {
    let cpu = run_test(0x8125, |c| {
        c.v[1] = 0x69;
        c.v[2] = 0x39
    });
    assert_eq!(cpu.v[0x1], 0x30, "SUBC should subtract correctly");
    assert_eq!(cpu.v[0xF], 0x01, "SUBC should set VF to 1 if Vx-Vy >= 0");
}

#[test]
fn test_shr() {
    let cpu = run_test(0x8106, |c| c.v[1] = 0x69);
    assert_eq!(cpu.v[1], 0x34, "SHR should shift correctly");
    assert_eq!(cpu.v[15], 0x1, "SHR should set VF correctly");
}

#[test]
fn test_sbn_coff() {
    let cpu = run_test(0x8127, |c| {
        c.v[1] = 0x69;
        c.v[2] = 0x39
    });
    assert_eq!(cpu.v[1], 0xD0, "SBN should underflow the result");
    assert_eq!(cpu.v[15], 0x00, "SBN should set VF to 0 if Vy-Vx < 0");
}

#[test]
fn test_sbn_con() {
    let cpu = run_test(0x8127, |c| {
        c.v[1] = 0x39;
        c.v[2] = 0x69
    });
    assert_eq!(cpu.v[1], 0x30, "SBN should subtract correctly");
    assert_eq!(cpu.v[15], 0x01, "SBN should set VF to 1 if Vy-Vx >= 0");
}

#[test]
fn test_shl() {
    let cpu = run_test(0x810E, |c| c.v[1] = 0x69);
    assert_eq!(cpu.v[1], 0xD2, "SHL should shift correctly");
    assert_eq!(cpu.v[15], 0x0, "SHL should set VF correctly");
}

#[test]
fn test_snev_eq() {
    let cpu = run_test(0x9010, |c| {
        c.v[0] = 0x69;
        c.v[1] = 0x69
    });
    assert_eq!(cpu.pc, 0x202, "PC shouldn't increment if Vx == Vy");
}

#[test]
fn test_snev_ne() {
    let cpu = run_test(0x9010, |c| {
        c.v[0] = 0x69;
        c.v[1] = 0x39
    });
    assert_eq!(cpu.pc, 0x204, "PC should increment if Vx != Vy");
}

#[test]
fn test_ldi() {
    let cpu = run_test(0xA420, |_| {});
    assert_eq!(cpu.index, 0x420, "I should change");
}

#[test]
fn test_jpv() {
    let cpu = run_test(0xB420, |c| c.v[0] = 0x69);
    assert_eq!(cpu.pc, 0x489, "PC should jump correctly");
}

#[test]
fn test_rnd() {
    let cpu = run_test(0xC607, |_| {});
    assert!(cpu.v[6] < 8, "RND mask should get applied");
}

#[test]
fn test_drw_basic() {
    let cpu = run_test(0xD105, |c| {
        c.load_memory(&[]);
        c.v[1] = 60
    });
    assert_eq!(
        cpu.frame_buffer[0], 0xF,
        "DRW should draw a built-in character correctly"
    );
    assert_eq!(
        cpu.v[0xF], 0,
        "VF should not be set since no collision occured"
    );
}

#[test]
fn test_drw_overflow() {
    let cpu = run_test(0xD105, |c| {
        c.load_memory(&[]);
        c.v[0] = 64;
        c.v[1] = 124
    });
    assert_eq!(cpu.frame_buffer[0], 0xF, "X and Y should overflow");
    assert_eq!(
        cpu.v[0xF], 0,
        "VF should not be set since no collision occured"
    );
}

#[test]
fn test_drw_collision() {
    let cpu = run_test(0xD105, |c| {
        c.load_memory(&[]);
        c.v[1] = 60;
        c.frame_buffer[0] = 0xF
    });
    assert_eq!(cpu.frame_buffer[0], 0, "XOR should be applied correctly");
    assert_eq!(cpu.v[0xF], 1, "VF should be set since a collision occured");
}

#[test]
fn test_lddt() {
    let cpu = run_test(0xF007, |c| {
        c.delay_timer = 0x39;
    });
    assert_eq!(cpu.v[0], 0x39, "DT should be changed correctly");
}

#[test]
fn test_ldvdt() {
    let cpu = run_test(0xF007, |c| {
        c.delay_timer = 0x39;
    });
    assert_eq!(cpu.v[0], 0x39, "V0 should be changed correctly");
}

#[test]
fn test_lddtv() {
    let cpu = run_test(0xF015, |c| {
        c.v[0] = 0x39;
    });
    assert_eq!(cpu.delay_timer, 0x39, "DT should be changed correctly");
}

#[test]
fn test_ldst() {
    let cpu = run_test(0xF018, |c| {
        c.v[0] = 0x39;
    });
    assert_eq!(cpu.sound_timer, 0x39, "ST should be changed correctly");
}

#[test]
fn test_addi() {
    let cpu = run_test(0xF01E, |c| {
        c.v[0] = 0x39;
        c.index = 0xFFF;
    });
    assert_eq!(cpu.index, 0x38, "I should be added and overflow");
}

#[test]
fn test_ldf() {
    let cpu = run_test(0xF029, |c| c.v[0] = 0x1);
    assert_eq!(cpu.index, 0x5, "I should be modified correctly");
}

#[test]
fn test_ldb() {
    let cpu = run_test(0xF033, |c| {
        c.v[0] = 0x7B;
        c.index = 0x420
    });
    assert_eq!(
        cpu.memory[0x420], 1,
        "Hundreds digit should be stored correctly"
    );
    assert_eq!(
        cpu.memory[0x421], 2,
        "Tens digit should be stored correctly"
    );
    assert_eq!(
        cpu.memory[0x422], 3,
        "Units digit should be stored correctly"
    );
}

#[test]
fn test_ldiv() {
    let cpu = run_test(0xF255, |c| {
        c.v[0] = 0x39;
        c.v[1] = 0x42;
        c.v[2] = 0x67;
        c.index = 0x420
    });
    assert_eq!(
        cpu.memory[0x420], 0x39,
        "V0 digit should be stored correctly"
    );
    assert_eq!(cpu.memory[0x421], 0x42, "V1 should be stored correctly");
    assert_eq!(cpu.memory[0x422], 0x67, "V2 should be stored correctly");
}

#[test]
fn test_ldvi() {
    let cpu = run_test(0xF265, |c| {
        c.memory[0x420] = 0x39;
        c.memory[0x421] = 0x42;
        c.memory[0x422] = 0x67;
        c.index = 0x420
    });
    assert_eq!(cpu.v[0], 0x39, "V0 digit should be stored correctly");
    assert_eq!(cpu.v[1], 0x42, "V1 should be stored correctly");
    assert_eq!(cpu.v[2], 0x67, "V2 should be stored correctly");
}
