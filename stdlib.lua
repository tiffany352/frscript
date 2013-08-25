local stdlib = {}

stdlib['+'] = function(_, ...)
    local acc = 0
    for _, v in pairs{...} do
        acc = acc + v
    end
    return acc
end

function stdlib.list(_, ...) return {...} end

function stdlib.let(ctx, name, val)
    ctx.atoms[name] = val
    return val
end

function stdlib.eval(ctx, label, ...)
    if type(label) == "function" then
        return label(ctx, ...)
    elseif ctx.env[label] then
        return ctx.env[label](...)
    end
end

function stdlib.var(ctx, label)
    if ctx.atoms[label] then
        return ctx.atoms[label]
    end
    return label
end

return stdlib

