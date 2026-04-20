# AI Compile Report

## Metadata
- Source: `../examples/source/main.hopping`
- Target: `stm32f403`
- Status: `success`
- Provider: `deepseek`

## IR Explanation
- DeepSeek AI 不可用，已降级到规则解释。
- 原因：read deepseek response failed: error decoding response body
- 规则解释：- 当前 IR 片段共 `24` 行。
- 赋值语句会映射为 `MOV` 或算术/比较指令。
- 发现 `ifz`：表示条件寄存器为 0 时跳转。
- 发现 `goto`：对应无条件跳转。
- 发现 `return`：会写入 `retval` 并停机。

## Source Excerpt
```hopping
func int main() {
    int a;
    int b;
    int c;
    a = 4;
    b = 3;
    c = a + b * 2;
    if (c > 5) {
        c = c - 1;
    } else {
        c = c + 1;
    }
    while (c > 0) {
        c = c - 1;
    }
    return c;
}
```

## IR Excerpt
```text
function main
  a = 4
  b = 3
  t0 = b * 2
  t1 = a + t0
  c = t1
  t2 = c > 5
  ifz t2 goto L0
  t3 = c - 1
  c = t3
  goto L1
  label L0
  t4 = c + 1
  c = t4
  label L1
  label L2
  t5 = c > 0
  ifz t5 goto L3
  t6 = c - 1
  c = t6
  goto L2
  label L3
  return c
end

```

> 建议仅作为辅助，请以编译器真实产物为准。

## Suggested Tests
_provider: `deepseek`_

- DeepSeek AI 不可用，使用规则建议。
- 原因：read deepseek response failed: error decoding response body

- 用例1：最小 happy path，覆盖声明/赋值/return。
- 用例3：分支与循环边界，确认跳转目标与终止条件。
- 用例4：非法输入（未知符号/错拼关键字）验证报错质量。