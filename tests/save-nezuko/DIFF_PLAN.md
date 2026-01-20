# 测试桩差异分析与修复计划

## 测试桩结构总结

### 文件列表
| 文件 | module name | 枚举数 | Bean数 | Table数 |
|------|-------------|--------|--------|---------|
| attribute.xml | attribute | 0 | 1 | 0 |
| battle.xml | battle | 0 | 7 | 2 |
| enums.xml | enums | 5 | 0 | 0 |
| forge.xml | forge | 2 | 3 | 1 |
| lottery.xml | lottery | 0 | 4 | 1 |
| resource.xml | resource | 0 | 1 | 0 |
| roblox.xml | (空) | 0 | 4 | 0 |
| role.xml | role | 0 | 5 | 1 |
| roll-skill.xml | rollSkill | 0 | 1 | 1 |
| skill.xml | skill | 0 | 1 | 1 |
| trait.xml | trait | 0 | 1 | 1 |
| weapon.xml | weapon | 0 | 2 | 1 |

## 关键差异

### 1. 枚举 alias 问题

**数值枚举需要 alias**（enums.xml）：
- `ItemType`: Role→alias="role", Consumable→alias="consumable", Currency→alias="currency"
- `SkillStyle`: Attack→alias="attack", Defense→alias="defense", Support→alias="support", Control→alias="control", Heal→alias="heal"
- `AttributeType`: Health→alias="health", Attack→alias="attack", Speed→alias="speed", CdReduction→alias="cdReduction", Critical→alias="critical", AttackSpeed→alias="attackSpeed", DamageBonus→alias="damageBonus", Luck→alias="luck", CoinBonus→alias="coinBonus"
- `CurrencyType`: Coin→alias="coin", Stone→alias="stone", Soul→alias="soul"

**字符串枚举不需要 alias**：
- `QualityType`: 值是字符串 (common, uncommon, rare, epic, legendary, mythic)
- `PieceType`: 值是字符串 (basic, legendary, mythical)
- `PieceAttributeType`: 有 tags="string"，值是字符串

### 2. module comment

测试桩使用自定义 comment，当前输出使用固定 "自动生成的 ts class Bean 定义"。

**解决方案**：在 TS 文件中添加模块级 JSDoc 注释，或接受当前输出格式。

### 3. ResourceConfig 字段可选性

测试桩：
```xml
<var name="icon" type="string?" comment="资源图标"/>
<var name="description" type="string?" comment="资源描述"/>
<var name="quality" type="enums.QualityType?" comment="资源品质"/>
```

需要在 TS 中将这些字段标记为可选。

### 4. lottery.xml 类型差异

测试桩：
- `CostConfig.single`: `type="map,string,int"` (无 sep)
- `CostConfig.multi10`: `type="map,string,int"` (无 sep)

当前输出可能有 `#sep=,|`，需要移除。

### 5. battle.xml 列表类型

测试桩：
- `AttackDetail.Points`: `type="list,AttackPointInfo"` (无 sep)
- `AllianceAttackInfo.Attacks`: `type="list,AttackInfo"` (无 sep)

当前输出可能有 `#sep=;`，需要移除。

### 6. skill.xml 列表类型

测试桩：
- `tags`: `type="(list#sep=|),string"`
- `styles`: `type="(list#sep=|),enums.SkillStyle"`

需要保留 sep。

### 7. roblox.xml 特殊属性

测试桩有 `valueType="1"` 和 `sep=","` 属性，这是特殊的 Luban 配置，可能需要单独处理或跳过。

## 修复计划

### Phase 1: 更新 TS 枚举文件（添加 @alias）

需要修改的文件：
1. `enums/item-type.ts` - 添加 @alias="role", @alias="consumable", @alias="currency"
2. `enums/skill-style.ts` - 添加 @alias="attack", @alias="defense", @alias="support", @alias="control", @alias="heal"
3. `enums/attribute-type.ts` - 添加 @alias="health", @alias="attack", @alias="speed", @alias="cdReduction", @alias="critical", @alias="attackSpeed", @alias="damageBonus", @alias="luck", @alias="coinBonus"
4. `enums/currency-type.ts` - 添加 @alias="coin", @alias="stone", @alias="soul"

### Phase 2: 更新 TS Bean 文件

1. `resource/resource-config.d.ts` - 将 icon, description, quality 改为可选
2. `lottery/cost-config.ts` - 移除 @mapsep
3. `battle/*.ts` - 移除不需要的 @sep
4. `weapon/*.ts` - 确保类型正确 (int, float)

### Phase 3: 处理特殊文件

1. `roblox.xml` - 可能需要手动创建或跳过（包含 valueType 等特殊属性）

### Phase 4: 验证

运行转换并对比所有文件。

## 执行顺序

```
Phase 1 (并行)
├── Agent A: 修改 enums/*.ts 添加 @alias
└── Agent B: 修改 resource/resource-config.d.ts

Phase 2 (并行)
├── Agent C: 修改 lottery/*.ts
├── Agent D: 修改 battle/*.ts
└── Agent E: 修改 weapon/*.ts

Phase 3 (串行)
└── 处理 roblox.xml

Phase 4 (串行)
├── 运行转换
├── 对比验证
└── 修复剩余差异
```
