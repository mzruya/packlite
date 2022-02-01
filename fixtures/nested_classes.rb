class A
  class B
    class C
      class DD::EE
        def in_d
          InD
          B::InD
        end
      end

      def in_c
        InC
      end
    end


    def in_b
      InB
    end
  end


  def in_a
    InA
  end
end

A::B::C.inspect_nesting