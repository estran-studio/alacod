def sub(p, old, new):
    s=open(p).read(); assert old in s,(p,old); open(p,'w').write(s.replace(old,new,1))
def suball(p, old, new):
    s=open(p).read(); assert old in s,(p,old); open(p,'w').write(s.replace(old,new))
R='std::env::var("X6D_REG").as_deref() != Ok'
Y='std::env::var("X6D_REG").as_deref() == Ok'
g='crates/game/src/character/enemy/ai/grab.rs'
# anciens interrupteurs
sub(g,"riders.get(&carrier.0).copied().unwrap_or(0) < rule.max_riders",f"({Y}(\"riders\") || riders.get(&carrier.0).copied().unwrap_or(0) < rule.max_riders)")
suball(g,"hold_slow(&mut modifiers, net_id.0, rule.slow, now.saturating_add(1));",f"if {R}(\"slow\") {{ hold_slow(&mut modifiers, net_id.0, rule.slow, now.saturating_add(if {Y}(\"expire\") {{ 1000 }} else {{ 1 }})); }}")
sub(g,"Some(_) if now >= riding.until =>",f"Some(_) if now >= riding.until && {R}(\"timeout\") =>")
sub(g,"Some((_, true, _)) => Some(Release::CarrierMelee),",f"Some((_, true, _)) if {R}(\"melee\") => Some(Release::CarrierMelee),\n                    Some((_, true, _)) => None,")
sub(g,"if struck {",f"if struck && {R}(\"melee\") {{")
sub(g,"} else if shakes >= rule.shake {",f"}} else if shakes >= rule.shake && {R}(\"shake\") {{")
# nouveaux
sub(g,"None | Some((_, _, true)) => Some(Release::Carrier),",f"None | Some((_, _, true)) if {R}(\"carrier\") => Some(Release::Carrier),\n                    None | Some((_, _, true)) => None,")
sub(g,"until: now.saturating_add(rule.telegraph),",f"until: now.saturating_add(if {Y}(\"crouch\") {{ 0 }} else {{ rule.telegraph }}),")
sub(g,"if let Some(every) = rule.bite_every.filter(|every| *every > 0) {",f"if let Some(every) = rule.bite_every.filter(|every| *every > 0 && {R}(\"bite\")) {{")
sub(g,"                    runtime.grab = GrabPhase::Ride;\n",f"                    runtime.grab = GrabPhase::Ride;\n                    if {Y}(\"bitenow\") {{ runtime.grab_bite = Some((now, carrier.clone())); }}\n")
sub(g,"                        commands.entity(entity).remove::<Riding>();\n                        runtime.grab = GrabPhase::Dazed {",f"                        commands.entity(entity).remove::<Riding>();\n                        if {Y}(\"wall\") {{ let mut moved = transform.clone(); moved.translation.x += bevy_fixed::fixed_math::new(30.0); commands.entity(entity).insert(moved); }}\n                        runtime.grab = GrabPhase::Dazed {{")
sub('crates/sim_core/src/grab.rs',"if now.saturating_sub(self.last_shake) > window {",f"if now.saturating_sub(self.last_shake) > window && {R}(\"window\") {{")
inp='crates/game/src/character/player/input.rs'
sub(inp,"if let Some(sway) = riding.sway_at(frame.frame) {",f"if let Some(sway) = riding.sway_at(frame.frame).filter(|_| {R}(\"sway\")) {{")
sub(inp,"if riding.is_some_and(|riding| riding.carrier.0 == net_id.0) {",f"if riding.is_some_and(|riding| riding.carrier.0 == net_id.0) && {R}(\"soft\") {{")
sub(inp,"velocity.main = dash_state.dash_direction * (step / dt);",f"velocity.main = dash_state.dash_direction * (step / dt);\n            if {Y}(\"dash\") && riders.iter().any(|(_, r)| r.carrier.0 == net_id.0) {{ velocity.main = velocity.main * fixed_math::FIXED_HALF; }}")
sub('crates/game/src/character/enemy/ai/pathing.rs',"        if let Some(riding) = riding {",f"        if let Some(riding) = riding.filter(|_| {R}(\"snap\")) {{")
sub('crates/game/src/model3d/playback.rs',"input.grab = grab_pose(runtime, behaviors, f);",f"input.grab = grab_pose(runtime, behaviors, f).filter(|_| {R}(\"pose\"));")
sub('crates/combat/src/weapons/mod.rs',"|| target_riding.is_some_and(|riding| riding.carrier.0 == bullet.source.0)",f"|| target_riding.is_some_and(|riding| (riding.carrier.0 == bullet.source.0 && {R}(\"bullets\")) || {Y}(\"allbullets\"))")
r='crates/game/src/character/enemy/ai/rules.rs'
sub(r,"grabbing: runtime.grab != GrabPhase::Idle,",f"grabbing: runtime.grab != GrabPhase::Idle && {R}(\"retain\"),")
sub(r,"""            GrabPhase::Leap { target, .. } => BehaviorMotion {
                direction: (fixed_math::FixedVec2::new(target.0, target.1) - position)""",f"""            GrabPhase::Leap {{ target: frozen, .. }} => BehaviorMotion {{
                direction: (if {Y}("aim") {{ target.unwrap_or(fixed_math::FixedVec2::new(frozen.0, frozen.1)) }} else {{ fixed_math::FixedVec2::new(frozen.0, frozen.1) }} - position)""")
b='crates/behaviors/src/lib.rs'
sub(b,"ctx.grabbing || (ctx.grab_ready && distance.is_some_and(|d| d <= *reach))",f"ctx.grabbing || (ctx.grab_ready && distance.is_some_and(|d| d <= *reach || {Y}(\"reach\")))")
