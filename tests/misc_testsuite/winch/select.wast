(module
  ;; Auxiliary
  (func $dummy)
  (table $tab funcref (elem $dummy))
  (memory 1)

  (func (export "select-i32") (param i32 i32 i32) (result i32)
    (select (local.get 0) (local.get 1) (local.get 2))
  )
  (func (export "select-i64") (param i64 i64 i32) (result i64)
    (select (local.get 0) (local.get 1) (local.get 2))
  )
  (func (export "select-f32") (param f32 f32 i32) (result f32)
    (select (local.get 0) (local.get 1) (local.get 2))
  )
  (func (export "select-f64") (param f64 f64 i32) (result f64)
    (select (local.get 0) (local.get 1) (local.get 2))
  )

  (func (export "as-select-first") (param i32) (result i32)
    (select (select (i32.const 0) (i32.const 1) (local.get 0)) (i32.const 2) (i32.const 3))
  )
  (func (export "as-select-mid") (param i32) (result i32)
    (select (i32.const 2) (select (i32.const 0) (i32.const 1) (local.get 0)) (i32.const 3))
  )
  (func (export "as-select-last") (param i32) (result i32)
    (select (i32.const 2) (i32.const 3) (select (i32.const 0) (i32.const 1) (local.get 0)))
  )

  (func (export "as-loop-first") (param i32) (result i32)
    (loop (result i32) (select (i32.const 2) (i32.const 3) (local.get 0)) (call $dummy) (call $dummy))
  )
  (func (export "as-loop-mid") (param i32) (result i32)
    (loop (result i32) (call $dummy) (select (i32.const 2) (i32.const 3) (local.get 0)) (call $dummy))
  )
  (func (export "as-loop-last") (param i32) (result i32)
    (loop (result i32) (call $dummy) (call $dummy) (select (i32.const 2) (i32.const 3) (local.get 0)))
  )

  (func (export "as-if-condition") (param i32)
    (select (i32.const 2) (i32.const 3) (local.get 0)) (if (then (call $dummy)))
  )
  (func (export "as-if-then") (param i32) (result i32)
    (if (result i32) (i32.const 1) (then (select (i32.const 2) (i32.const 3) (local.get 0))) (else (i32.const 4)))
  )
  (func (export "as-if-else") (param i32) (result i32)
    (if (result i32) (i32.const 0) (then (i32.const 2)) (else (select (i32.const 2) (i32.const 3) (local.get 0))))
  )

  (func (export "as-br_if-first") (param i32) (result i32)
    (block (result i32) (br_if 0 (select (i32.const 2) (i32.const 3) (local.get 0)) (i32.const 4)))
  )
  (func (export "as-br_if-last") (param i32) (result i32)
    (block (result i32) (br_if 0 (i32.const 2) (select (i32.const 2) (i32.const 3) (local.get 0))))
  )

  (func $f (param i32) (result i32) (local.get 0))

  (func (export "as-call-value") (param i32) (result i32)
    (call $f (select (i32.const 1) (i32.const 2) (local.get 0)))
  )
  (func (export "as-return-value") (param i32) (result i32)
    (select (i32.const 1) (i32.const 2) (local.get 0)) (return)
  )
  (func (export "as-drop-operand") (param i32)
    (drop (select (i32.const 1) (i32.const 2) (local.get 0)))
  )
  (func (export "as-br-value") (param i32) (result i32)
    (block (result i32) (br 0 (select (i32.const 1) (i32.const 2) (local.get 0))))
  )
  (func (export "as-local.set-value") (param i32) (result i32)
    (local i32) (local.set 0 (select (i32.const 1) (i32.const 2) (local.get 0))) (local.get 0)
  )
  (func (export "as-local.tee-value") (param i32) (result i32)
    (local.tee 0 (select (i32.const 1) (i32.const 2) (local.get 0)))
  )
  (global $a (mut i32) (i32.const 10))
  (func (export "as-global.set-value") (param i32) (result i32)
    (global.set $a (select (i32.const 1) (i32.const 2) (local.get 0)))
    (global.get $a)
  )

  (func (export "as-unary-operand") (param i32) (result i32)
    (i32.eqz (select (i32.const 0) (i32.const 1) (local.get 0)))
  )
  (func (export "as-binary-operand") (param i32) (result i32)
    (i32.mul
      (select (i32.const 1) (i32.const 2) (local.get 0))
      (select (i32.const 1) (i32.const 2) (local.get 0))
    )
  )
  (func (export "as-test-operand") (param i32) (result i32)
    (block (result i32)
      (i32.eqz (select (i32.const 0) (i32.const 1) (local.get 0)))
    )
  )

  (func (export "as-compare-left") (param i32) (result i32)
    (block (result i32)
      (i32.le_s (select (i32.const 1) (i32.const 2) (local.get 0)) (i32.const 1))
    )
  )
  (func (export "as-compare-right") (param i32) (result i32)
    (block (result i32)
      (i32.ne (i32.const 1) (select (i32.const 0) (i32.const 1) (local.get 0)))
    )
  )
)

(assert_return (invoke "select-i32" (i32.const 1) (i32.const 2) (i32.const 1)) (i32.const 1))
(assert_return (invoke "select-i64" (i64.const 2) (i64.const 1) (i32.const 1)) (i64.const 2))
(assert_return (invoke "select-f32" (f32.const 1) (f32.const 2) (i32.const 1)) (f32.const 1))
(assert_return (invoke "select-f64" (f64.const 1) (f64.const 2) (i32.const 1)) (f64.const 1))

(assert_return (invoke "select-i32" (i32.const 1) (i32.const 2) (i32.const 0)) (i32.const 2))
(assert_return (invoke "select-i32" (i32.const 2) (i32.const 1) (i32.const 0)) (i32.const 1))
(assert_return (invoke "select-i64" (i64.const 2) (i64.const 1) (i32.const -1)) (i64.const 2))
(assert_return (invoke "select-i64" (i64.const 2) (i64.const 1) (i32.const 0xf0f0f0f0)) (i64.const 2))

(assert_return (invoke "select-f32" (f32.const nan) (f32.const 1) (i32.const 1)) (f32.const nan))
(assert_return (invoke "select-f32" (f32.const nan:0x20304) (f32.const 1) (i32.const 1)) (f32.const nan:0x20304))
(assert_return (invoke "select-f32" (f32.const nan) (f32.const 1) (i32.const 0)) (f32.const 1))
(assert_return (invoke "select-f32" (f32.const nan:0x20304) (f32.const 1) (i32.const 0)) (f32.const 1))
(assert_return (invoke "select-f32" (f32.const 2) (f32.const nan) (i32.const 1)) (f32.const 2))
(assert_return (invoke "select-f32" (f32.const 2) (f32.const nan:0x20304) (i32.const 1)) (f32.const 2))
(assert_return (invoke "select-f32" (f32.const 2) (f32.const nan) (i32.const 0)) (f32.const nan))
(assert_return (invoke "select-f32" (f32.const 2) (f32.const nan:0x20304) (i32.const 0)) (f32.const nan:0x20304))

(assert_return (invoke "select-f64" (f64.const nan) (f64.const 1) (i32.const 1)) (f64.const nan))
(assert_return (invoke "select-f64" (f64.const nan:0x20304) (f64.const 1) (i32.const 1)) (f64.const nan:0x20304))
(assert_return (invoke "select-f64" (f64.const nan) (f64.const 1) (i32.const 0)) (f64.const 1))
(assert_return (invoke "select-f64" (f64.const nan:0x20304) (f64.const 1) (i32.const 0)) (f64.const 1))
(assert_return (invoke "select-f64" (f64.const 2) (f64.const nan) (i32.const 1)) (f64.const 2))
(assert_return (invoke "select-f64" (f64.const 2) (f64.const nan:0x20304) (i32.const 1)) (f64.const 2))
(assert_return (invoke "select-f64" (f64.const 2) (f64.const nan) (i32.const 0)) (f64.const nan))
(assert_return (invoke "select-f64" (f64.const 2) (f64.const nan:0x20304) (i32.const 0)) (f64.const nan:0x20304))

(assert_return (invoke "as-select-first" (i32.const 0)) (i32.const 1))
(assert_return (invoke "as-select-first" (i32.const 1)) (i32.const 0))
(assert_return (invoke "as-select-mid" (i32.const 0)) (i32.const 2))
(assert_return (invoke "as-select-mid" (i32.const 1)) (i32.const 2))
(assert_return (invoke "as-select-last" (i32.const 0)) (i32.const 2))
(assert_return (invoke "as-select-last" (i32.const 1)) (i32.const 3))

(assert_return (invoke "as-loop-first" (i32.const 0)) (i32.const 3))
(assert_return (invoke "as-loop-first" (i32.const 1)) (i32.const 2))
(assert_return (invoke "as-loop-mid" (i32.const 0)) (i32.const 3))
(assert_return (invoke "as-loop-mid" (i32.const 1)) (i32.const 2))
(assert_return (invoke "as-loop-last" (i32.const 0)) (i32.const 3))
(assert_return (invoke "as-loop-last" (i32.const 1)) (i32.const 2))

(assert_return (invoke "as-if-condition" (i32.const 0)))
(assert_return (invoke "as-if-condition" (i32.const 1)))
(assert_return (invoke "as-if-then" (i32.const 0)) (i32.const 3))
(assert_return (invoke "as-if-then" (i32.const 1)) (i32.const 2))
(assert_return (invoke "as-if-else" (i32.const 0)) (i32.const 3))
(assert_return (invoke "as-if-else" (i32.const 1)) (i32.const 2))

(assert_return (invoke "as-br_if-first" (i32.const 0)) (i32.const 3))
(assert_return (invoke "as-br_if-first" (i32.const 1)) (i32.const 2))
(assert_return (invoke "as-br_if-last" (i32.const 0)) (i32.const 2))
(assert_return (invoke "as-br_if-last" (i32.const 1)) (i32.const 2))

(assert_return (invoke "as-call-value" (i32.const 0)) (i32.const 2))
(assert_return (invoke "as-call-value" (i32.const 1)) (i32.const 1))
(assert_return (invoke "as-return-value" (i32.const 0)) (i32.const 2))
(assert_return (invoke "as-return-value" (i32.const 1)) (i32.const 1))
(assert_return (invoke "as-drop-operand" (i32.const 0)))
(assert_return (invoke "as-drop-operand" (i32.const 1)))
(assert_return (invoke "as-br-value" (i32.const 0)) (i32.const 2))
(assert_return (invoke "as-br-value" (i32.const 1)) (i32.const 1))
(assert_return (invoke "as-local.set-value" (i32.const 0)) (i32.const 2))
(assert_return (invoke "as-local.set-value" (i32.const 1)) (i32.const 1))
(assert_return (invoke "as-local.tee-value" (i32.const 0)) (i32.const 2))
(assert_return (invoke "as-local.tee-value" (i32.const 1)) (i32.const 1))
(assert_return (invoke "as-global.set-value" (i32.const 0)) (i32.const 2))
(assert_return (invoke "as-global.set-value" (i32.const 1)) (i32.const 1))

(assert_return (invoke "as-unary-operand" (i32.const 0)) (i32.const 0))
(assert_return (invoke "as-unary-operand" (i32.const 1)) (i32.const 1))
(assert_return (invoke "as-binary-operand" (i32.const 0)) (i32.const 4))
(assert_return (invoke "as-binary-operand" (i32.const 1)) (i32.const 1))
(assert_return (invoke "as-test-operand" (i32.const 0)) (i32.const 0))
(assert_return (invoke "as-test-operand" (i32.const 1)) (i32.const 1))
(assert_return (invoke "as-compare-left" (i32.const 0)) (i32.const 0))
(assert_return (invoke "as-compare-left" (i32.const 1)) (i32.const 1))
(assert_return (invoke "as-compare-right" (i32.const 0)) (i32.const 0))
(assert_return (invoke "as-compare-right" (i32.const 1)) (i32.const 1))
