local lpeg = require "lpeg"

local frs = {}
local P, S, R, B, C, Carg, Cb, Cc, Cf, Cg, Cs, Ct, Cmt, Cp, V = lpeg.P, lpeg.S, lpeg.R, lpeg.B, lpeg.C, lpeg.Carg, lpeg.Cb, lpeg.Cc, lpeg.Cf, lpeg.Cg, lpeg.Cs, lpeg.Ct, lpeg.Cmt, lpeg.Cp, lpeg.V

frs.rules = {
      ws = V'space'^0
    , sws = V'space'^1
    , digits = V'digit'^1
    , label = (V'alpha' + V'digit' + S"~!@#$%^&*_-+=/<>'")^1
    , number    = P"-"^-1 * V'digits' * (P"." * V'digits')^-1 * (S"Ee" * P'-'^-1 * V'digits')^-1
                + P"0x" * V'xdigit'^1
    , string    = P"\"" * (1 - P"\\\"")^0 * P"\""
    , literal   = V'label'
                + V'string'
                + V'number'
    , sexpr = "(" * V'ws' * V'label' * (V'ws' * V'expr')^0 * V'ws' * ")"
    , expr  = V'sexpr'
            + V'literal'
}
lpeg.locale(frs.rules)

function frs.fill(t, t2)
    for k, v in pairs(t2) do
        if not t[k] then
            t[k] = v
        end
    end
    return t
end

local stdlib
function frs.context()
    stdlib = stdlib or require 'stdlib'
    local atoms = {}
    setmetatable(atoms, {__index=stdlib})
    local ctx = {env = frs, atoms = atoms}
    local captures = {
        V'expr'
        , sexpr = frs.rules.sexpr/function(...)return stdlib.eval(ctx,...) end
        , number = frs.rules.number/tonumber
        , label = C(frs.rules.label)/function(...)return stdlib.var(ctx, ...) end
    }
    local grammar = P(frs.fill(captures, frs.rules))
    ctx.exec = function(str)
        return grammar:match(str)
    end
    ctx.interactive = function()
        while true do
            io.stdout:write("= ")
            local res = ctx.exec(io.stdin:read())
            print(frs.show(res))
        end
    end
    return ctx
end

function frs.fold(f, acc, t)
    for k, v in pairs(t) do
        acc = f(acc, k, v)
    end
    return acc
end

function frs.show(v, t, is_key)
    if not t then t = 0 end
    if type(v) == "number" then
        return is_key and string.format("[%d]", v) or tostring(v)
    elseif type(v) == "string" then
        local parser = P(frs.fill({frs.rules.label}, frs.rules))
        if is_key and C(parser):match(v) == v then 
            return v 
        else
            return "'"..v .."'"
        end
    elseif type(v) == "nil" then
        return 'nil'
    elseif type(v) == "function" then
        local name = debug.getinfo(v, 'n').name
        if name and is_key then
            return "[function "..name.."]"
        elseif is_key then
            return "[anonymous function]"
        else
            return tostring(v)..(name and (" ("..name..")") or "")
        end
    elseif type(v) == "table" then
        if is_key then
            return tostring(v)
        end
        local s = "("
        local last_key = 0
        local maxlen = 0
        local keys = {}
        for k, _ in pairs(v) do
            keys[k] = frs.show(k, t+1, true)
            if #keys[k] > maxlen then
                maxlen = #keys[k]
            end
        end
        for k, c in pairs(v) do
            if type(k) == "number" and last_key == k - 1 then
                s = string.format("%s\n%s%s", s, string.rep("\t", t+1), frs.show(c, t+1))
            else
                local pad = string.rep(" ", maxlen-#keys[k])
                s = string.format("%s\n%s%s: %s%s", s, string.rep("\t", t+1), keys[k], type(c) == "table" and "" or pad, frs.show(c, t+1))
            end
            last_key = k
        end
        return s.." )"
    elseif type(v) == "userdata" then
        return tostring(v)
    else
        return "NYI: "..type(v)
    end
end

return frs

