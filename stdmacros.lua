local stdmacros = {}

local frs
function stdmacros.ast(ctx, expr)
    frs = frs or require 'frscript'
    return {'literal', frs.show(expr)}
end

function stdmacros.fn(ctx, name, params, expr)
    ctx.atoms[name] = function(ctx, ...)
        local args = {}
        for i, v in pairs(params) do
            args[v] = ({...})[i]
        end
        return ctx.atoms.eval(ctx, expr, args)
    end
end

function stdmacros.let(ctx, name, val)
    ctx.atoms[name] = ctx.atoms.eval(ctx, val)
    return ctx.atoms[name]
end

return stdmacros

